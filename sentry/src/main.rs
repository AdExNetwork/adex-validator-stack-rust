#![deny(clippy::all)]
#![deny(rust_2018_idioms)]

use clap::{App, Arg};
use hyper::{Body, Error, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use adapter::{AdapterTypes, DummyAdapter, EthereumAdapter};
use primitives::adapter::{Adapter, AdapterOptions};
use primitives::config::configuration;
use primitives::util::tests::prep_db::{AUTH, IDS};

const DEFAULT_PORT: u16 = 8005;

#[tokio::main]
async fn main() {
    let cli = App::new("Sentry")
        .version("0.1")
        .arg(
            Arg::with_name("config")
                .help("the config file for the validator worker")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("adapter")
                .short("a")
                .help("the adapter for authentication and signing")
                .required(true)
                .default_value("ethereum")
                .possible_values(&["ethereum", "dummy"])
                .takes_value(true),
        )
        .arg(
            Arg::with_name("keystoreFile")
                .short("k")
                .help("path to the JSON Ethereum Keystore file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dummyIdentity")
                .short("i")
                .help("the identity to use with the dummy adapter")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("clustered")
                .short("c")
                .help("Run app in cluster mode with multiple workers"),
        )
        .get_matches();

    let environment = std::env::var("ENV").unwrap_or("development".into());
    let port = std::env::var("PORT")
        .map(|s| s.parse::<u16>().expect("Invalid port(u16) was provided"))
        .unwrap_or_else(|_| DEFAULT_PORT);
    let config_file = cli.value_of("config");
    let config = configuration(&environment, config_file).unwrap();
    let clustered = cli.is_present("clustered");

    let _adapter = match cli.value_of("adapter").unwrap() {
        "ethereum" => {
            let keystore_file = cli
                .value_of("keystoreFile")
                .expect("keystore file is required for the ethereum adapter");
            let keystore_pwd = std::env::var("KEYSTORE_PWD").unwrap();

            let options = AdapterOptions {
                keystore_file: Some(keystore_file.to_string()),
                keystore_pwd: Some(keystore_pwd),
                dummy_identity: None,
                dummy_auth: None,
                dummy_auth_tokens: None,
            };
            let ethereum_adapter = EthereumAdapter::init(options, &config).expect("Should initialize ethereum adapter");

            AdapterTypes::EthereumAdapter(Box::new(ethereum_adapter))
        }
        "dummy" => {
            let dummy_identity = cli
                .value_of("dummyIdentity")
                .expect("Dummy identity is required for the dummy adapter");

            let options = AdapterOptions {
                dummy_identity: Some(dummy_identity.to_string()),
                // @TODO: this should be prefilled using fixtures
                dummy_auth: Some(IDS.clone()),
                dummy_auth_tokens: Some(AUTH.clone()),
                keystore_file: None,
                keystore_pwd: None,
            };

            let dummy_adapter = DummyAdapter::init(options, &config).expect("Should initialize dummy adapter");
            AdapterTypes::DummyAdapter(Box::new(dummy_adapter))
        }
        // @TODO exit gracefully
        _ => panic!("We don't have any other adapters implemented yet!"),
    };

    run(clustered, port).await;
}

//async fn run(config: Config, adapter: AdapterTypes, _clustered: bool, port: u16) {
async fn run(_clustered: bool, port: u16) {
    let addr = ([127, 0, 0, 1], port).into();

    let make_service = make_service_fn(|_| async {
        Ok::<_, Error>(service_fn(|req| async {
            Ok::<_, Error>(handle_routing(req))
        }))
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

fn handle_routing(req: Request<Body>) -> Response<Body> {
//fn handle_routing(req: Request<Body>, _config: Arc<Config>, _adapter: Arc<AdapterTypes>) -> Response<Body> {
    if req.uri().path().starts_with("/channel") {
        return Response::new(Body::from("Channel!!"));
    }

    not_found()
}

fn not_found() -> Response<Body> {
    let mut response = Response::new(Body::from("Not found"));
    let status = response.status_mut();
    *status = StatusCode::NOT_FOUND;
    response
}
