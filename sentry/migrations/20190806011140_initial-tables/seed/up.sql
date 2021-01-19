INSERT INTO
channels (id, creator, deposit_asset, deposit_amount, valid_until, targeting_rules, spec, exhausted)
VALUES
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0x033ed90e0fec3f3ea1c9b005c724d704501e0196',
    '0x89d24A6b4CcB1B6fAA2625fE562bDD9a23260359',
    '1000000000000000000000',
    to_timestamp(4102444800),
    '[]',
    '{"targeting_rules": [], "minPerImpression":"1","maxPerImpression":"10","created":1564383600000,"pricingBounds":{"CLICK":{"min":"0","max":"0"}},"withdrawPeriodStart":4073414400000,"validators":[{"id":"0xce07CbB7e054514D590a0262C93070D838bFBA2e","url":"http://localhost:8005","fee":"100"},{"id":"0xC91763D7F14ac5c5dDfBCD012e0D2A61ab9bDED3","url":"http://localhost:8006","fee":"100"}]}',
    '{}'
);


INSERT INTO
validator_messages (channel_id, "from", msg, received)
VALUES
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"Heartbeat","timestamp":"2021-01-18T13:33:14.651Z","signature":"0xed1ce8c85fa0cc0357f95aac7e90de509bc0af0903f50bfc7a35240773dbd71e5cc9663acb814e1f272f97da3c2a9b39615c34f055eb6cbf1623c36792d993801c","stateRoot":"909ff01345470dcbcdaf82f0c7f2047c5d36323a440fd34b7928f26019669ab9"}',
    to_timestamp(1610980480)
),
(
    '0x061d5e2a67d0a9a10f1c732bca12a676d83f79663a396f7d87b3e30b9b411088',
    '0xce07CbB7e054514D590a0262C93070D838bFBA2e',
    '{"type":"NewState","balances":{"0x1E810Bc144EBA2C4acd9f0FFa1b07525A9c1F704":"42480633000000000000","0x5d62321228bC75936dd29f9c129f8DbDfcB93264":"9990060000000000000","0x9b88bcc2824FC929DbF646A9A42F2E34cF0BAbbC":"3964404000000000000","0xd5860D6196A4900bf46617cEf088ee6E6b61C9d6":"15099294000000000000","0x5874Bb0Fa607202bAbCE73a0235aaC8bDc78c342":"10501932000000000000","0x984020DdBcFA53B040BCc68f8e9187Ff6C44cC7D":"1607784000000000000","0xC21a1Fd8E2af71A7c18CC07742B9f84afBA1AB15":"5298210000000000000","0xC0841555DBE77D20FCc53C593697D48986C0a576":"5872299000000000000","0x3E0c2D0dB53E29a7e11d5b9774cE6448DC52ebdC":"9706782000000000000","0xA1229f7A064e6a51bd0B4997d2BD4D551BACdFc3":"241893000000000000"},"stateRoot":"bead8616d10aa90fe562a95a610173ccdf8c766edcf1c0ec0be89aba6467770c","signature":"0xece04ee2cfb42ac1582ba3728c60bc3252bd371d735e3a97a84c74985cd4dd96090cc172c1caa7b9229903d4f4366bdd26945532cb2f0d74dc8a2b63f43e38d11b","exhausted":false}',
    to_timestamp(1610980480)
);
