#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use adex_primitives::market_channel::{MarketChannel, MarketStatusType};
use adex_primitives::{AdUnit, BigNum, SpecValidators, TargetingTag};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub type TargetingScore = f64;
pub type MinTargetingScore = TargetingScore;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdViewManagerOptions {
    // Defaulted via defaultOpts
    #[serde(rename = "marketURL")]
    pub market_url: String,
    /// Defaulted
    pub accepted_states: Vec<MarketStatusType>,
    /// Defaulted
    pub min_per_impression: BigNum,
    /// Defaulted
    pub min_targeting_score: MinTargetingScore,
    /// Defaulted
    pub randomize: bool,
    pub publisher_addr: String,
    pub whitelisted_token: String,
    pub whitelisted_type: Option<String>,
    /// Defaulted
    pub top_by_price: usize,
    /// Defaulted
    pub top_by_score: usize,
    #[serde(default)]
    pub targeting: Vec<TargetingTag>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub fallback_unit: Option<String>,
    /// Defaulted
    pub disabled_video: bool,
}

#[derive(Debug)]
pub struct UnitByPrice {
    pub unit: AdUnit,
    pub channel_id: String,
    pub validators: SpecValidators,
    pub min_targeting_score: MinTargetingScore,
    pub min_per_impression: BigNum,
}

#[derive(Debug)]
pub struct Unit {
    pub unit: AdUnit,
    pub channel_id: String,
    pub validators: SpecValidators,
    pub min_targeting_score: MinTargetingScore,
    pub min_per_impression: BigNum,
    pub targeting_score: TargetingScore,
}

impl Unit {
    pub fn new(by_price: UnitByPrice, targeting_score: TargetingScore) -> Self {
        Self {
            unit: by_price.unit,
            channel_id: by_price.channel_id,
            validators: by_price.validators,
            min_targeting_score: by_price.min_targeting_score,
            min_per_impression: by_price.min_per_impression,
            targeting_score,
        }
    }
}

pub fn apply_selection(campaigns: &[MarketChannel], options: AdViewManagerOptions) -> Vec<Unit> {
    let eligible = campaigns.iter().filter(|campaign| {
        options
            .accepted_states
            .contains(&campaign.status.status_type)
            && campaign
                .spec
                .active_from
                .map(|datetime| datetime < Utc::now())
                .unwrap_or(true)
            && campaign.deposit_asset == options.whitelisted_token
            && campaign.spec.min_per_impression >= options.min_per_impression
    });

    let mut units: Vec<UnitByPrice> = eligible
        .flat_map(|campaign| {
            let mut units = vec![];
            for ad_unit in campaign.spec.ad_units.iter() {
                let unit = UnitByPrice {
                    unit: ad_unit.clone(),
                    channel_id: campaign.id.clone(),
                    validators: campaign.spec.validators.clone(),
                    min_targeting_score: ad_unit
                        .min_targeting_score
                        .or(campaign.spec.min_targeting_score)
                        .unwrap_or(0.into()),
                    min_per_impression: campaign.spec.min_per_impression.clone(),
                };

                units.push(unit);
            }

            units
        })
        .collect();

    // Sort
    units.sort_by(|b, a| a.min_per_impression.cmp(&b.min_per_impression));
    units.truncate(options.top_by_price);

    let units = units.into_iter().filter(|unit| {
        options
            .whitelisted_type
            .as_ref()
            .map(|whitelisted_type| {
                whitelisted_type != &unit.unit.ad_type
                    && !(options.disabled_video && is_video(&unit.unit))
            })
            .unwrap_or(false)
    });

    let mut by_score: Vec<Unit> = units
        .collect::<Vec<UnitByPrice>>()
        .into_iter()
        .filter_map(|by_price| {
            let targeting_score =
                calculate_target_score(&by_price.unit.targeting, &options.targeting);
            if targeting_score >= options.min_targeting_score
                && targeting_score >= by_price.min_targeting_score
            {
                Some(Unit::new(by_price, targeting_score))
            } else {
                None
            }
        })
        .collect();
    by_score.sort_by(|a, b| {
        a.targeting_score
            .partial_cmp(&b.targeting_score)
            .expect("Should always be comparable")
    });
    by_score.truncate(options.top_by_score);

    by_score
}

fn is_video(ad_unit: &AdUnit) -> bool {
    ad_unit.media_mime.split('/').collect::<Vec<&str>>()[0] == "video"
}

fn calculate_target_score(a: &[TargetingTag], b: &[TargetingTag]) -> TargetingScore {
    a.iter()
        .map(|x| -> TargetingScore {
            match b.iter().find(|y| y.tag == x.tag) {
                Some(b) => (&x.score * &b.score).into(),
                None => 0.into(),
            }
        })
        .sum()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Event {
    #[serde(rename = "type")]
    event_type: String,
    publisher: String,
    ad_unit: String,
}

#[derive(Serialize)]
struct EventBody {
    events: Vec<Event>,
}

pub fn get_html(
    options: &AdViewManagerOptions,
    ad_unit: AdUnit,
    channel_id: &str,
    validators: &SpecValidators,
) -> String {
    let ev_body = EventBody {
        events: vec![Event {
            event_type: "IMPRESSION".into(),
            publisher: options.publisher_addr.clone(),
            ad_unit: ad_unit.ipfs.clone(),
        }],
    };

    let on_load_code: String = validators.into_iter().map(|validator| {
        let fetch_opts = "{ method: 'POST', headers: { 'content-type': 'application/json' }, body: this.dataset.eventBody }";
        let fetch_url = format!("{}/channel/{}/events", validator.url, channel_id);

        format!("fetch({}, {});", fetch_url, fetch_opts)
    }).collect();

    let ev_body = serde_json::to_string(&ev_body).expect("should convert");

    get_unit_html(&options, ad_unit, &ev_body, &on_load_code)
}

fn get_unit_html(
    options: &AdViewManagerOptions,
    ad_unit: AdUnit,
    event_body: &str,
    on_load: &str,
) -> String {
    let size = options
        .width
        .and_then(|width| options.height.and_then(|height| Some((width, height))));

    let style_size = size
        .map(|(width, height)| format!("width: {}; height: {};", width, height))
        .unwrap_or("".to_string());
    let element_html = if is_video(&ad_unit) {
        // get_video_html
        // videoHtml({ evBody, onLoadCode, size, imgUrl, mediaMime: unit.mediaMime })
        "videoHtml"
    } else {
        // get_image_html
        // imageHtml({ evBody, onLoadCode, size, imgUrl }))
        "imageHtml"
    };

    let adex_icon = "<a href=\"https://www.adex.network\" target=\"_blank\" rel=\"noopener noreferrer\"
            style=\"position: absolute; top: 0; right: 0;\"
        >
		<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" x=\"0px\" y=\"0px\" width=\"18px\"
			height=\"18px\" viewBox=\"0 0 18 18\" style=\"enable-background:new 0 0 18 18;\" xml:space=\"preserve\">
			<style type=\"text/css\">
				.st0{fill:#FFFFFF;}
				.st1{fill:#1B75BC;}
			</style>
			<defs>
			</defs>
			<rect class=\"st0\" width=\"18\" height=\"18\"/>
			<path class=\"st1\" d=\"M14,12.1L10.9,9L14,5.9L12.1,4L9,7.1L5.9,4L4,5.9L7.1,9L4,12.1L5.9,14L9,10.9l3.1,3.1L14,12.1z M7.9,2L6.4,3.5
				L7.9,5L9,3.9L10.1,5l1.5-1.5L10,1.9l-1-1L7.9,2 M7.9,16l-1.5-1.5L7.9,13L9,14.1l1.1-1.1l1.5,1.5L10,16.1l-1,1L7.9,16\"/>
   			</svg>
		</a>";

    let result = format!("
        <div style=\"position: relative; overflow: hidden; {size}\">
            <a href=\"{target_url}\" target=\"_blank\" rel=\"noopener noreferrer\">{element_html}</a>
            {adex_icon}
        </div>
    ", target_url = ad_unit.target_url, size = style_size, element_html = element_html, adex_icon = adex_icon);

    result.to_string()
}
