# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

[5c58196](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5c5819620edc2916142e0e1de7e2e46540879728)...[a7a333f](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a7a333f74dba06232d96d37593411df7b5f6c018)

### Bug Fixes

- Use REAL_HOME and add sudo-safe function execution ([`b55d353`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b55d35369e2ee30062943b759c457b49dca13afc))
- Replace single brackets with double brackets for improved condition checks ([`72af293`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/72af293e5b21f193cd45ead32ab3990507ec7fc5))
- Fix code formatting ([`8929d60`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8929d60d596893d177828d81c7d82114454b671d))
- Fix merge conflict ([`7cb2644`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7cb26441b38b0f291a3f13e2b8a643fcb5358c6c))
- Update default APP_VERSION to 0.5.0-rc.10 in installation scripts ([`5b47cb0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5b47cb088fecce503c11ca4be52023097bc21bac))
- Add push trigger for fix/update-scripts branch in workflow ([`a6b9aba`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a6b9aba90a66a75c042085c4523ad533b25cb509))
- Update push trigger branch to fix/update-feature in workflow ([`4ef9f64`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4ef9f64639017eaa9044aefd77490f19c77c513b))
- Centralize OS guard into shared utils and apply to all platform scripts ([`1d960a1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1d960a10dc4db97983d1874abb785d323d0d8524)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Fix update process ([`4fe583f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4fe583fb24ac330a2e849d77340102ab99cb520d))
- Fix ui flinking and update failures ([`0599a18`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0599a1847b49f8c86e5e7e355dccef81c59dbed4))
- Fix code formatting ([`a9bacd7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a9bacd710764adc57c601dd380bd7a60ad56e3b9))
- Fix failing ci ([`2b70730`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2b70730401c9e15a6facc5019508ffdee6ad5daa))
- Fix code formatting ([`acd8406`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/acd8406d1b6f74de71dd89f3d3ec9cfb7b699ec8))
- Fix failing ci ([`7824de4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7824de4a5213fac8821bb28f15b15821fc9f5b46))
- Enforce failure in release workflow when WAZUH_GATEWAY_URL secret is missing ([`6d7b679`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6d7b67965175a64fa04869c8cceea5bb5556f1d2))
- Specify bash shell for Gateway URL configuration step in release workflow ([`1c3ae93`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1c3ae93e0d396f8ccc090d371606219e1dac4e33))
- Update macOS build configuration to use specific self-hosted runners ([`ab945ce`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ab945ced1173972c8c68886a6a4b8449a4bcec9c))
- Simplify Windows build configuration by removing explicit target and adjusting paths ([`54205c3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/54205c38aa5211aa4f16c793b2326a228ca0becf))
- Update macOS build configuration to use correct self-hosted runner labels ([`1a2b18a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1a2b18a0c2a6c025eb5cb2d8ef7f5b390170e601))
- Skip version updates in release workflow for pre-release tags to maintain MSI compatibility ([`b589032`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b5890325eb723b7025b5025b08f9cfb089dfbb3a))
- Add type attributes to buttons for accessibility and improve CI permissions ([`5feb39e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5feb39e7838de44a3f8447879d608b47e773d992)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Replace npm install with npm ci for consistent dependency installation in Tauri client builds ([`35ea81c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/35ea81c334c6640328f0a102197317d1b8c07107)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Reload macOS client tray app after update (#181) ([`8280801`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/828080186ac8d5eee1dfc04bf497b372be2360a4)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* add debug logs to troubleshoot update failures, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* fix: update temporary branch reference in scripts for testing, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* refactor: remove deep debug logging and update branch references in macOS scripts, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* revert checksums for macOS scripts, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* Address review comments from @t-desmond and @mbiti2, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* docs: update CHANGELOG.md and checksums [skip ci]

* docs: update CHANGELOG.md and checksums [skip ci]

* chore: remove temporary push trigger for checksum update, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

* docs: update CHANGELOG.md and checksums [skip ci]

---------, Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>, Co-authored-by:GitHub Action <action@github.com>
- Update repository reference from 'user-main' to 'main' in installation scripts ([`e927335`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e927335b8e0d407b262e0e03dfba1eefd37740ea)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Documentation

- Update CHANGELOG.md and checksums [skip ci] ([`12ba1de`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/12ba1de8d6b6e82f8ea3856f0844c6e8575d81fc))
- Update CHANGELOG.md and checksums [skip ci] ([`2159ab9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2159ab9bb5617bc25714eebd4c7b30ad91b3b876))
- Update CHANGELOG.md and checksums [skip ci] ([`9f27cb8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9f27cb86f12d03dd645143fb0faada6835d72213))
- Update CHANGELOG.md and checksums [skip ci] ([`20ae207`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/20ae2078433ed027e757524da8cd24d592a208ed))
- Update CHANGELOG.md and checksums [skip ci] ([`c4c428a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c4c428aad636ac93668cf2859ab7b0e4b73b46b6))
- Update CHANGELOG.md and checksums [skip ci] ([`a89a71d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a89a71d2dd02e0943e8c0b69e0d4ff41474af98c))
- Update CHANGELOG.md and checksums [skip ci] ([`ec8c62a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ec8c62aba9a97aa931c79aa68209597010a5ed8e))
- Update CHANGELOG.md and checksums [skip ci] ([`5537fed`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5537fed0319a16b0dbddaa551c8e08cf39d3cce3))
- Update CHANGELOG.md and checksums [skip ci] ([`fb820c8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fb820c80a5418cc131becf80cd9baae63fe0dd3b))
- Update CHANGELOG.md and checksums [skip ci] ([`1be4466`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1be446633edd8465f32fbe83e78f44a014b7a202))
- Update CHANGELOG.md and checksums [skip ci] ([`5d91a90`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5d91a90428b235bb681a29e4223d47d30237f02a))
- Update CHANGELOG.md and checksums [skip ci] ([`7ee4c7a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7ee4c7a05af2b909c8d01adfef0fa5fd2b693064))
- Update CHANGELOG.md and checksums [skip ci] ([`105b3bb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/105b3bbe70e1c1e7541fe855d9ea5f111a813107))
- Update CHANGELOG.md and checksums [skip ci] ([`4fe2a33`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4fe2a33160b01dee2c7294b7294f4a1431c6b576))
- Update CHANGELOG.md and checksums [skip ci] ([`b17b36a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b17b36aeefd96534714635277b779170d6142467))
- Update CHANGELOG.md and checksums [skip ci] ([`6cc5591`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6cc55918b5b2417f2fffd55443ec158ac47dac05))
- Update CHANGELOG.md and checksums [skip ci] ([`7485401`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/748540184b99516058095893f221f12d2b0cbf6d))
- Update CHANGELOG.md and checksums [skip ci] ([`f783e1e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f783e1e748b524a5209dd25950caaa6f6c6c261d)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update CHANGELOG.md and checksums [skip ci] ([`95b4fd5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/95b4fd5bc1ff9790e0b4668b9b95a5d289673723))
- Update CHANGELOG.md and checksums [skip ci] ([`5a924d7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5a924d70486cd8f5c41696b817247acb91f56505))
- Update CHANGELOG.md and checksums [skip ci] ([`79d39c3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/79d39c38ffc7ccb821602d59c5d3328f786d7bf5))
- Update CHANGELOG.md and checksums [skip ci] ([`61f2285`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/61f2285b498909e47fb2715c2ac415ec6c5ba877))
- Update CHANGELOG.md and checksums [skip ci] ([`1a24c02`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1a24c021bbdab6727500e52f308be674d8f9233a))
- Update CHANGELOG.md and checksums [skip ci] ([`1b3fde1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1b3fde11bd8a0e8812d42149a0e92e1f7d31391d))
- Update CHANGELOG.md and checksums [skip ci] ([`13f7d0a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/13f7d0ad987490b90f452c5598d0dfca85cb44ec))
- Update CHANGELOG.md and checksums [skip ci] ([`bb72cea`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bb72cea4e5752970f75f1c0b6348f37177ca5616))
- Update CHANGELOG.md and checksums [skip ci] ([`d8f111c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d8f111cf5fad3372c906573a2788088e21284c7a))
- Update CHANGELOG.md and checksums [skip ci] ([`0dc41a3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0dc41a337d444d701c45b409650cf42edeafa1b3))
- Update CHANGELOG.md and checksums [skip ci] ([`e408aab`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e408aabfe8fc7df8c976dc81b5866f74e6a4d5e3))
- Update CHANGELOG.md and checksums [skip ci] ([`0b43051`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0b430519a830a19ce1be0fb9306d0ac235f5422e))
- Update CHANGELOG.md and checksums [skip ci] ([`c9a978e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c9a978e8f148e97979274fc76cfc58e55d1a02da))
- Update CHANGELOG.md and checksums [skip ci] ([`a0098c7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a0098c7f622d21cf22e4b44ae577c4fe758ea989))
- Update CHANGELOG.md and checksums [skip ci] ([`92b1f29`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/92b1f298a29fd2454d16ae7e68c15bacbf15f536))
- Update CHANGELOG.md and checksums [skip ci] ([`6a9a5bb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6a9a5bbd1ac7f817cad5bb579ee85bde761374f8))
- Update CHANGELOG.md and checksums [skip ci] ([`94055eb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/94055ebc21d018e1980da78eb9602d860a097b01))
- Update CHANGELOG.md and checksums [skip ci] ([`6c628a8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6c628a80a57232e1e8af62b12d5b78d63c53b8c7))
- Update CHANGELOG.md and checksums [skip ci] ([`68f31ac`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68f31ac494d19f6faef30f7da2c5be46ac1daa4e))
- Update CHANGELOG.md and checksums [skip ci] ([`4cfc7d6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4cfc7d6c6e2fd3405cf8c180afca2cf8e8591202))
- Update CHANGELOG.md and checksums [skip ci] ([`d3a3b66`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d3a3b667ab851aab5a01357dac2d934e1b363553))
- Update CHANGELOG.md and checksums [skip ci] ([`8c64768`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8c64768ffe8dcbb583f1a824cd77fc9aad203eaa))
- Update CHANGELOG.md and checksums [skip ci] ([`7579869`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/75798691c911c4f632b4bcb4718b8c34a42c858a))
- Refactor README and add comprehensive feature documentation (#170) ([`cc6900f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cc6900f06a113146a1bdd04d05f36f626ee35bd2))
- Update CHANGELOG.md and checksums [skip ci] ([`c03691d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c03691d31670daf55e9f826067730a28e6efd973))
- Update CHANGELOG.md and checksums [skip ci] ([`a426333`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a4263339546831996632f734ce7d299ad2f95fd9))

### Features

- Implement Unix update logic ([`10b1c0b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/10b1c0b675e7248e259b48ca74d419efa317c9ca)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Decouple wazuh-agent repo reference from agent-status repo reference using configurable environment variables ([`655658b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/655658b6d3d937050134e650e6449d64e4741299)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Implement update completion status monitoring and add macOS testing documentation ([`122812d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/122812dae215b1a045df547d0e88034f6941557d))
- Add update modal component and integrate update status handling in UpdatesView ([`57418f6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/57418f64c952960ed8fead1879c1cc139dd10e4c)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Use sudo to save and manage setup scripts to handle permission restrictions ([`00fdbb2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/00fdbb2167f8711eca3db01b5589587703673963))
- Add SCA compliance dashboard with agent identity propagation ([`ce44e33`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ce44e339dc83fc05e1a2336901dc72743da70644)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add active-responses.log tailing to capture update progress for stable releases and fixed npm  package vulnerabilities ([`e9b687f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e9b687f0aa53cf635904f22b14703be91f14b4fb))
- Add SCA compliance dashboard with agent identity propagation ([`7df3cfc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7df3cfcee3fe11ed4ecfebe4ce2f7b3ccab23716)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add opencode workflow for PR review and issue tasks ([`47cb1b3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/47cb1b3f8e9b8021d7f5c1ea89f755bba2f4602a))
- Integrate AI-driven compliance remediation with secure and configuration support ([`176f199`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/176f199a0c159a46e8ec85c470fd56b02b863bd4)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add HMAC auth for SCA compliance requests ([`9a775b6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9a775b695e08ebc087f318ecb125c5a91f0df920)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add matrix-based cross-compilation support for Linux, MacOS, and Windows in release workflow ([`895472c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/895472c0e73f229cfb4405afc12730d67f081fa0))
- Implement responsive sidebar with static button styling ([`3fe2826`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3fe2826d44bc33e6a895541008890231b0f5affa))
- Enforce HTTPS for repository URLs in scripts and components and resolve SonaQube warnings ([`47328cd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/47328cd9d298f5d8f4522e74a547b22a2cfdc8f1)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Resolve macOS update UI freeze & update AI command allowlist (#175) ([`779c333`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/779c33310ba7e32525d970792d6462ab594c675a))
- Enhance update notification mechanism for macOS and Windows scripts ([`a7a333f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a7a333f74dba06232d96d37593411df7b5f6c018)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Miscellaneous Tasks

- APP_VERSION -> v0.4.2 ([`a9821d6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a9821d6d5cea76a737541ac47277633f15ab6827))
- Update default repository reference to user-main ([`d5d38ea`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d5d38ea7763ac2bd9d26c1421e425d2eba02096e)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add matrix-based rust workflow for server and client ([`1efbba5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1efbba5155133b1ac431c1cc61ee90300b2aabd5)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Configure cargo-machete ([`ed20f05`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ed20f053f812c7629824252633373607ec4f3cfd)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Pin CI action versions ([`86f8eed`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/86f8eed0780ec5b382cc89992e573c5febcfa693)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Explicitize stable Rust toolchain in release workflows ([`e772b8c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e772b8c96901efc4b936a25ca41afd787709adc9))
- Remove unused push trigger from scripts workflow ([`cba86f7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cba86f706e8baec58884efe664b5d93513d7115c)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update branding assets in wazuh-agent-status-rust-client index configuration ([`5371ef0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5371ef0ad8c3c2d233bcc0382af018cc5d32c12a))
- Updated wazuh version to 0.5.0-rc-12 ([`baff009`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/baff009535c74859941bb45355e6960ec0a37bf7))
- Update release workflow to trigger on tag creation ([`f1b36f7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f1b36f71c9f37fcf493664600d13cb5998a5ad6d))
- Update gateway_url to development environment endpoint ([`8c47868`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8c478688a4d2d07b9bfe793ac42049a622b5dcd2))
- Inject gateway url from github secrets during build ([`84545d9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84545d9578f951d010df96330cb4c614f20e024a))
- Fail release workflow when WAZUH_GATEWAY_URL secret is missing ([`528de52`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/528de5239660910832858c6dd31528f6f0f25402))
- Bump application version to 0.5.1 across all install and update scripts ([`7e4fe60`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7e4fe601f4b6a48829b6c5ac2968f3a70be0ff02))
- Trigger workflow ([`2c3368e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2c3368e495346ac8a6fda562ecef7db02957356f))
- Trigger workflow ([`6d00b7c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6d00b7cc95d4c0f31757f9467832cc019ea530b6))
- Adjust layout constraints for improved responsiveness ([`6c52319`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6c523192f9fb1b64a52167b8c37bfd9eb25303bd))
- Automate version bumping during releases ([`3830226`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3830226da8a31f9b3cbe90e5f30372f60f286471))
- Bump application version to 0.5.1-rc.1 ([`73835ab`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/73835abb0e3269fd6b10a30636952e638ba94dd3))
- Restrict release workflow trigger to specific project paths ([`6046d55`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6046d550d5e12a73c90bfb9f45d03c2e702eadb8))
- Reverted app version to 0.5.1 ([`f02e784`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f02e784342949c9030ebd33903452ebc4ed1507d))

### Performance

- Perform cleanup by removing dead code ([`106c4bf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/106c4bf6c2d5552db71bc0f788176898ab2d5b9f))
- Perform cleanup by removing dead code ([`b547b08`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b547b082c6ec5e70a89415e16fd0caf51111403d))

### Refactor

- Improve checksum validation logic in utils.ps1 and update GitHub Actions workflows ([`269a11f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/269a11f71ca0b0d2f38bc3398f6af30b77598313)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Simplify PowerShell scripts ([`82068d5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/82068d54b31cc85589803e9220cb9695a984ce38)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Standardize PowerShell scripts, improve LogsView UI robustness ([`ba1bec6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ba1bec67a21fc1d471a96f17170bbb1fa1650e03)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Streamline update process and improve logging in agent manager ([`8cd9b03`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8cd9b0387c59ad743260031deed7364d4dd069e1))
- Improve repository reference handling in PowerShell scripts ([`7a0c65b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7a0c65be81e71fbc47ec6741fc11c0d6a2ead14d)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Modernize Rust code with let-else/if-let chains, update Cargo edition ([`629f59a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/629f59a2cbd701bb41cb1653557d8beb82b24ac6)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Format rust code ([`ba81dee`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ba81deee4e3aadfea13d355504288e91c141f995))
- Remove unused APP_VERSION variable from Windows scripts ([`c838a39`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c838a39c7e6e851604b39b33e991f31288840b8d)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Centralize branding module, add light mode, configurable colors ([`323e010`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/323e01022d063f9c76fbe141ad372457ab75b301)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Reformat sudo script writing logic and imports for improved readability ([`a1c9647`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a1c9647a776879fd57958965a62c5d9f2e75748e))
- Simplify setup script saving logic and add Windows support ([`fc1f8db`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fc1f8db436c31d15974badf0fc0f94f7a32a4152))
- Simplify error creation in manager process wait logic using std::io::Error::other ([`bef9dd6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bef9dd65fbc3a69024159e563a11ca9c1efc19ca))
- Improve formatting of error handling for child process output ([`4ee4a70`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4ee4a7016824b5cc2781996e2831234fd49bac9b))
- Update brand identifier constant in rust-client index file ([`e3ac49f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e3ac49f4a885019cf7b6dd07e32d22bf82cff94f))
- Simplify UpdateModal icon rendering logic and update type imports ([`99da827`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/99da8277e1dc0c966576a288482f3c0cd5e622ce))
- Update brand styling and module configuration in index.ts ([`2d7b21c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2d7b21c5250bbc2c7faf5a0e8bf2f8258ea48629))
- Replace hardcoded colors in ErrorBoundary with CSS custom properties ([`44fd84c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/44fd84cba742701b9b6ab6d22c6338beadd6dc60))
- Simplify comments and improve clarity in opencode workflow ([`bf422c1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bf422c195609eb464c7fa59b753aa30763e2d25e))
- Addressed sonarQube issues ([`fc8df6d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fc8df6d580eb8cf9af09b5a53c8ef6edfc1ba542))
- Modernize CSS text wrapping and modularize ComplianceView parser ([`d17da10`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d17da10116b05bd5fe49fa9a3068333d83749818)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Remove unused files and consolidate code for Wazuh agent status ([`8ccd604`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8ccd604f19336ec584db79d550bdcda48b73e8a0)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update client components, improve agent metrics, and add compliance utilities ([`f7f6c7a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f7f6c7a1ec6671c3300c0682775c84621e5bfd20)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Simplify check icon rendering in ComplianceView component ([`ce53b62`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ce53b629caaa9cbdd36cbb53a77dc70f6c8483a8)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Styling

- Update dismiss button background and remove shimmer animation ([`ca6d565`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ca6d5651d7af4bea151ce6861509a11ca31ce2b1))

### Testing

- Add comprehensive unit tests for AI keychain configuration persistence and status tracking ([`79ac802`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/79ac802f86dc6933037fca24e11ff4782a0742b1)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

## 0.4.2.rc1-user - 2025-07-16

[dd23cea](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dd23cea91c3bd8623e70983b24c80ddefeae3328)...[5c58196](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5c5819620edc2916142e0e1de7e2e46540879728)

### Bug Fixes

- Update command arguments for service status checks in darwin, linux, and windows scripts ([`5fe0e3d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5fe0e3dd78eacaf24da29146cd4584cb9829c610))

## 0.3.3 - 2025-07-14

[e76f7ad](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e76f7ad75a0cc77ce21ae3d83e9b97c248fdf068)...[dd23cea](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dd23cea91c3bd8623e70983b24c80ddefeae3328)

### Bug Fixes

- Improve process handling, binary locking, and TLS security during installation and uninstallation ([`dee028d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dee028d397a70898d688d8ae5fa4fcc820f9c57d))
- Remove unnecessary blank line in Remove-Binaries function ([`ab9a379`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ab9a3797009ade118f6b46df4a0a6e8a9c0d288f))
- Update script URLs to reference specific version tags ([`4c135e7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4c135e727d3a3ac56db111eee886222b777d9800))
- Update adorsys-update script URL to use raw GitHub content link ([`67b1141`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/67b11413cff43d0093757dff4a432985e7530d84))
- Update BASE_URL in install script to correct release path ([`95134ff`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/95134ff57211588cb2a7f3bf784d97c955912f3c))
- Refactor sudo command and control path usage in darwin and linux scripts ([`0b126ca`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0b126ca9f1987a02af82f9a23d67e7154d50dd5a))
- Add sed alternative function for compatibility with gsed ([`06a2889`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/06a288951bbb6d8544640c80ef4042cbb708ce01))
- Correct icon path for macOS and standardize warning message function name ([`42fdb78`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/42fdb7877df30c95f2bf9f22f71e79d54a94ece1))

### Documentation

- Update CHANGELOG.md and checksums [skip ci] ([`3739fb3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3739fb3581052c56d711c421dc7b0321b4271fbf))
- Update CHANGELOG.md and checksums [skip ci] ([`60dee42`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/60dee4221f8c18b1f90391409f64ad802c863d84))
- Update CHANGELOG.md and checksums [skip ci] ([`8a4e01e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8a4e01e2c505b7cb3b1b4ef262b376b79b88b70e))
- Update CHANGELOG.md and checksums [skip ci] ([`d6508f0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d6508f00231f6984d8bbf6cbd9b227d9c422cbdb))

### Features

- Configure workflow triggers for script paths on push events ([`ae951c5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ae951c5267ef9803d692e1704d843c834e41ab4a))
- Add vendored OpenSSL to server, improve file lock handling during updates, and refine installation and uninstallation process logic. ([`6e9a763`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6e9a7636a56b594a50944911ef0dcbd69ce28f4c))
- Rename "Tray App Version" to "Wazuh Agent Setup Version" ([`bc9df36`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc9df3666767231b3b30a30a4ceb3d0d55ceccfa))

### Miscellaneous Tasks

- Touch scripts to trigger workflow ([`22a96ea`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/22a96ea4501e3c5c2d3bb2265e5a2abd34323e59))

## 0.5.0 - 2026-05-19

[10bf8a5](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/10bf8a54a3541924dd69afe09220c66947bed0c4)...[e76f7ad](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e76f7ad75a0cc77ce21ae3d83e9b97c248fdf068)

### Bug Fixes

- Update binary download URL to use specific version tag and remove redundant script logic ([`e76f7ad`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e76f7ad75a0cc77ce21ae3d83e9b97c248fdf068))

## 0.5.0-rc.9 - 2026-05-19

[86d7dd4](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/86d7dd4664c240979969bd5c1d9fd47ee14a0e41)...[10bf8a5](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/10bf8a54a3541924dd69afe09220c66947bed0c4)

### Features

- Add openssl dependency with vendored features to Cargo.toml ([`10bf8a5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/10bf8a54a3541924dd69afe09220c66947bed0c4))

## 0.5.0-rc.8 - 2026-05-19

[9c8d4ed](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9c8d4ed22f6105784efc0777f5cdcc322a8ad9a6)...[86d7dd4](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/86d7dd4664c240979969bd5c1d9fd47ee14a0e41)

## 0.5.0-rc.7 - 2026-05-18

[e2386ad](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e2386ad9b1fcee87b9085e937fcff7b5a7f35788)...[9c8d4ed](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9c8d4ed22f6105784efc0777f5cdcc322a8ad9a6)

### Bug Fixes

- Fix failing ci ([`d673fd4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d673fd4a8578fbbde793fb38a298f7e8a742f7bf))

### Documentation

- Update CHANGELOG.md and checksums [skip ci] ([`cfb18ef`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cfb18ef0c6c0bf130373c35c86c5fc273c7d9553))
- Update CHANGELOG.md and checksums [skip ci] ([`48d414b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/48d414bceb8aa061408a5a6e3940c45baf240001))
- Update CHANGELOG.md and checksums [skip ci] ([`5aa38d0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5aa38d06dcd7ac2cf788a10b0bcc02bb67c3ff1d))
- Update CHANGELOG.md and checksums [skip ci] ([`56c1e43`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/56c1e433f5a832a6a05c0d2a93edcb950bb5e810))
- Update CHANGELOG.md and checksums [skip ci] ([`d88fc43`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d88fc432f073290172367f071a90915a8da1eebf))

### Miscellaneous Tasks

- Restrict release workflow triggers to specific subdirectories ([`9e1bdfe`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9e1bdfe061a782519e2ab35c4cd163a02688b4b8))
- Remove path filters from release workflow trigger in order to create a release ([`9c8d4ed`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9c8d4ed22f6105784efc0777f5cdcc322a8ad9a6))

### Refactor

- Replace error_exit with error_message/return and include icon in release artifacts ([`68c5ecf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68c5ecf9e38b531dc69a6fc595c5879e9690ca20))
- Replace bash conditional expressions with test command in log file and directory checks ([`268a560`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/268a560bd360fe9103c67addb4d7de4805f9099c))

### Build

- Update Linux runner to 22.04 and configure musl target for static binary build ([`fe18441`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fe18441b84ea3ff3905e7b45fb570142d6d5c410))

## 0.5.0-rc.6 - 2026-05-18

[d5d8f6a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d5d8f6a35abdd8b9f176fbabe973c3764cddab4a)...[e2386ad](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e2386ad9b1fcee87b9085e937fcff7b5a7f35788)

### Documentation

- Update CHANGELOG.md and checksums [skip ci] ([`d157642`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d1576428e618fb7f0aad2800693aaccd5703d7dc))

### Refactor

- Replace legacy test commands with bash-native double brackets in utility scripts ([`f3321df`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f3321df8b83ff5349760aa8c43a5a71da787f74b)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add explicit return statements and simplify sudoers validation logic in utils.sh ([`fce01dc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fce01dc0ceca74169f944454decbec989a892153)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

## 0.5.0-rc.5 - 2026-05-18

[b6752d8](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b6752d831752af621c9ee87d6516fcb9d712544a)...[d5d8f6a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d5d8f6a35abdd8b9f176fbabe973c3764cddab4a)

### Bug Fixes

- Replace wazuh-control status with sysinfo process check in is_agent_running to avoid lock file race condition, and Replace wazuh-control info with direct VERSION.json read in get_agent_version. ([`dd57b28`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dd57b2812ddfd98efd87dace8a03b69e1af70cc3))
- Correct BASE_URL string replacement and add target variable for macOS launchctl check ([`6957e03`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6957e03f0a81cae876d94d8e21c9dde78580f440)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update default WAZUH_GROUP to wheel for macOS installations ([`870fdc0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/870fdc0a05f4cc8fc83683da77655678abcc7254)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update sudoers file ownership to GID 0 for cross-platform compatibility ([`e60d114`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e60d114f9ff9eb5c1f1900b0bdac1ceff0e0d05e))
- Use double brackets for file existence check in install script ([`cd70dce`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cd70dcefc7c1d39b9f49827640c76f46790c034e))

### Documentation

- Update CHANGELOG.md and checksums [skip ci] ([`d0bc6b8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d0bc6b88dbe177e67365422ae327b9fb036dd82c))
- Update CHANGELOG.md and checksums [skip ci] ([`fa37a31`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fa37a313032d7cec0effc2b495262b555f08fafb))
- Update CHANGELOG.md and checksums [skip ci] ([`6b022a1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6b022a17c9068216495b3ec5c4c4c203a618584b))
- Update CHANGELOG.md and checksums [skip ci] ([`39691b3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/39691b3f1ec69f9df0e905ebab1c64779f251e67))
- Update CHANGELOG.md and checksums [skip ci] ([`0143a53`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0143a53f8bb2ad0fdf73fe8c7daa33fc64bd70bb))
- Update CHANGELOG.md and checksums [skip ci] ([`b531086`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b531086acdefc9c2b8a4a040a41d8920951a82fa))
- Update CHANGELOG.md and checksums [skip ci] ([`83e8df5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/83e8df541f89de11124e3af3a86b9249352e2e28))
- Update CHANGELOG.md and checksums [skip ci] ([`256bf9d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/256bf9d82b621b2f4f7166eb0e8f95cea37e4397))
- Update CHANGELOG.md and checksums [skip ci] ([`1d917fa`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1d917fabd2ec816f95e13725a8303930e716bd67))
- Update CHANGELOG.md and checksums [skip ci] ([`65986f4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/65986f480c39e7f594df482eed0005314d736b45))
- Update CHANGELOG.md and checksums [skip ci] ([`d5d8f6a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d5d8f6a35abdd8b9f176fbabe973c3764cddab4a))

### Features

- Improve download URL validation and strengthen checksum verification logic in installation scripts ([`82244eb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/82244eb145ca74ae117e30385873f217844815b8)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add setup_sudoers function to grant passwordless execution for wazuh-control ([`7953d8a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7953d8aea992fc86125460f4e408d0e42730c70b)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add desktop application entry and icon installation to Linux install and uninstall scripts ([`0198d89`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0198d89dd9b0ef6e622c01498d90b051ba6bf483))
- Add agent status indicators to tray menu and implement navigation to update screen ([`7848768`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7848768f099869dda6090b1a33961c77bdbaeeca))
- Add dynamic status indicators to tray icon and optimize log scrolling behavior ([`9d1543b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9d1543b7cace7e86fb65b79c113a7cdc945fbd18)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Miscellaneous Tasks

- Bump version to 0.5.0-rc.4 and improve configuration loading diagnostics and error messaging ([`ef0ff6d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ef0ff6dfbf64a021f6a0703ef7ee695042350d8f))
- Downgrade APP_VERSION to 0.5.0-rc.3 across all installation and update scripts ([`ecaa5b9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ecaa5b9a26cbb95a489ef3a599a2c0e62b226f5f))
- Bump version to 0.4.3 and pin repo refs to release tag across all platforms ([`54f68f5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/54f68f5ef3b9652291d813a2568a7ba0e3b4698e))

### Refactor

- Standardize bash usage, improve user detection, and update release workflow logic ([`c6352d7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c6352d78e88d3fdbc06ef78212df7319ba6e14fb)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Remove unified install script, update repo references, and add sudoers file cleanup to uninstallers ([`d2a29f6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d2a29f62c9adcdf878f2206cb6650b2850429319)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Standardize sed_inplace utility and remove redundant sudo calls in installation scripts ([`2582fd5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2582fd5f19b7696b83841812b4f4ce201e82dc82)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Improve sed_inplace compatibility on macOS by using temporary backup files and add input validation ([`2b57e8d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2b57e8d3d39f13a4b52e73e21818a237545ba79d))
- Migrate shell scripts to use double brackets and bump application version to 0.5.0-rc.4 ([`06302ab`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/06302ab94e6f0e1810e0a0db980aa931359c7c16)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

## 0.5.0-rc.2 - 2026-05-12

[5940735](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/59407352eeaecf8fe4dbb527747b387aafca4d2c)...[b6752d8](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b6752d831752af621c9ee87d6516fcb9d712544a)

### Features

- Add legacy Go component cleanup and migration tracking to Linux and macOS installers ([`1559382`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/155938246a3326cfe8c0b25955652c25c5ae5ee5)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Configure log file path via environment variables and ensure log directory existence and permissions ([`3be5fe6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3be5fe66b47d6985dff5a51537d44b6bedf341a2)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Miscellaneous Tasks

- Add workflow trigger for feature branch and sort script checksums by filename ([`68cdf54`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68cdf546a9ffd90d446addddc78ad3a79cb31ded)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update WAZUH_AGENT_STATUS_REPO_REF to dynamically use current branch reference ([`5a508db`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5a508db7076568f78fa5d06485aec4befd81e62c)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Bump application version to 0.5.0-rc.2 and update release workflow configuration ([`b6752d8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b6752d831752af621c9ee87d6516fcb9d712544a)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Refactor

- Remove installation profiles and update version to 0.5.0-rc.1 ([`c55bcfd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c55bcfd4ee21103367247bd7185ce77b74e46553)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Clean up CI workflows, simplify artifact handling, and update Linux installation user/group logic ([`39385ac`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/39385ac178e344d6943d37e359673381f3a3d5a2)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Move log file to a dedicated directory and update path configuration ([`86e0092`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/86e0092b0fd8cea060c9f7dd4b1c2a35660efcd2)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

## 0.5.0-rc.1 - 2026-05-08

[c9e1fe1](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c9e1fe1c521d1f5671c2cee7146a4e9841928c7a)...[5940735](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/59407352eeaecf8fe4dbb527747b387aafca4d2c)

### Bug Fixes

- Update release workflow and shell script utilities ([`7d768cf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7d768cf786959e7b7414360827f9761525f8d573))
- Corrected command_exists function to return actual command existence ([`f69e8ca`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f69e8ca48684f99f7176324bcca2a10ab593e554))
- Define install profile for windows uninstall script ([`a0d9d54`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a0d9d5452b8230ddeb6d69c47c36fbfa3b600489))
- Use REAL_HOME and add sudo-safe function execution ([`22bf5a6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/22bf5a6b7d5a6289ca63b95ffc99b56a8711d0c3))
- Fix sonaqube warnings ([`0582f21`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0582f21367b51fea53548d356bba3864c841e02d))
- Define missing WAZUH_AGENT_STATUS_REPO_URL in install.ps1 ([`9d8de89`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9d8de890566c2ebb866f6e3e867ea9a1f1e9e7f6))
- Update install.ps1 to use latest release fallback ([`05665c0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/05665c0c76130763a3535df574e7a4ca578031ed))
- Resolve HOME env var correctly in launchd context ([`0a069b7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0a069b7ec56c5f8601b014abeb5b6365cb4396d0))
- Update macos install script to run launchctl commands as the actual user and remove stale checksum entry ([`0c4f39c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0c4f39c0b9ff8572cbd3a0c88a0806d45c15e95c))
- Persist stream across navigation and history on connection ([`5d715ee`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5d715eeec069e06d25fa35c1a56c7f61d26c2252))
- Better windows errors, env override, remove idle timeout ([`a020eed`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a020eed76b0622a120876209cfdfc2f0d23947a2))
- Use ossec.log directly in agent root on windows ([`e44933c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e44933cfbde99a48fe3ba7ff85a03f12f72999b1))
- Use platform-specific self-healing restart commands ([`7af49f5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7af49f56e621b2b3ea1475a2147d57620831a6fb))
- Add diagnostics to server sync loop for Windows debugging ([`39f602f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/39f602fee2024839b71c8d96a50f3eb7cdb9008a))
- Correct process names for Wazuh agent metrics ([`1f292d7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1f292d78dc540333ed25e2552c6be9698d0c9acb))
- Fix(ci) added `runs-on` field for `update-changelog-and-checksums` job ([`a1ed19d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a1ed19d4b03910794d30aeeae24ae4148646f730))
- Corrected dtolnay/rust-action => dtolnay/rust-toolchain ([`c543146`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c54314654225ea7c8fe802e5489c784767713768))
- Corrected ubuntu dependency name ([`18904fb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/18904fb135518afd99665885ea2d51d9502e6fe1))
- Add missing dependencies for Linux build ([`dd31997`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dd31997e597d6cd0f35d45b7d0ec603cb20f620d))
- Add debug logs for tauri build ([`e208719`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e208719b08effa0ec673842f7a61ee4d4dd8a3b5))
- Added missing fuse deps for linux tauri build ([`8abbd39`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8abbd39a1074099f9b5e155e1152aad5846a8aea))
- Fix packaging process ([`247f4c4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/247f4c4df93c61e0c544db5c24359c1b0e29414a))
- Update Tauri build command to include deb and rpm bundles ([`b38c880`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b38c880b1ab1a077c27a522d19ed5f8ed46133f6))
- Improve app configuration loading and add resource path fallback ([`bac8c3b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bac8c3b3cc4fe450c9911ca0e65c34b537955369))
- Update macOS build configuration to use latest version ([`79b2c65`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/79b2c65d77460f0d8a60b1d7286c09d21ca2b308))
- Enhance error handling and clarify comments in AppConfig loading ([`ff5c12a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ff5c12a5f72478a552cd180da416ae3f4aac3ba3))

### Documentation

- Update CHANGELOG.md and checksums [skip ci] ([`677fc81`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/677fc8107c39a6930f369bcc941da24f221b3d9d))
- Update CHANGELOG.md and checksums [skip ci] ([`c5ce08d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c5ce08d015f7a8dab25c56bbb261b807d4463cf4))
- Update CHANGELOG.md and checksums [skip ci] ([`227d624`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/227d6243a622c19921f121a9513db3f34342be55))
- Update CHANGELOG.md and checksums [skip ci] ([`76b564a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/76b564add79af071556322fd16565da428d50aec))
- Update CHANGELOG.md and checksums [skip ci] ([`dcd1b0b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dcd1b0b20f94b9af9fe348d6fc093b6cafcec6c1))
- Update CHANGELOG.md and checksums [skip ci] ([`140531c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/140531c12c3a5812407bd4349cdf1971cd2402f7))
- Update agent enrollment documentation and images with release configuration adjustments ([`65b84bd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/65b84bdc9298a6e29d1fe031ff96a069b634dcb0)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update CHANGELOG.md and checksums [skip ci] ([`991c7e7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/991c7e7d964e2e85c794f9f1a3fd5fa21fd8e69a))
- Update CHANGELOG.md and checksums [skip ci] ([`5d43bf4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5d43bf42eb09a734d7e6ae1bc1d233b46005969c))
- Update CHANGELOG.md and checksums [skip ci] ([`d251627`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d251627b90b4e302d2ea6b296389d5ab5af2ace6))
- Update CHANGELOG.md and checksums [skip ci] ([`f698bfd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f698bfd5620d687b9b1301960284b4f8abcf8c30))
- Update CHANGELOG.md and checksums [skip ci] ([`ab78e8e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ab78e8ed46ad404e9255f6d4ca96fed073166342))
- Update CHANGELOG.md and checksums [skip ci] ([`3f51b26`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3f51b2675c2180ab7bd2321b14eae541566ac655))
- Update CHANGELOG.md and checksums [skip ci] ([`424999a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/424999aa16cfef19aaa0052f67ed7fa395b44652))
- Update CHANGELOG.md and checksums [skip ci] ([`da0277c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/da0277c9156213bd4d4368c98716f54ab1d4e5fe))
- Update CHANGELOG.md and checksums [skip ci] ([`f1d5f48`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f1d5f4840b14a828cf4dc047be8b7ecac5b51790))

### Features

- Added precomit ([`73fba0e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/73fba0e7da7ffb50eb5a92d16174a96299b1663f))
- Update macOS launchd service management to use kickstart for existing services and remove redundant unload calls ([`d985f42`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d985f42f3722efb98d390e7102dd522540efa3c5))
- Implement real-time ossec.log streamer ([`6b6ca51`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6b6ca51209e6d98cfb5011fb681fe46272528874))

### Miscellaneous Tasks

- Resolve merge conflicts and align with feat/self-healing-and-update branch ([`58c417e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/58c417eb7a0c6a87dfecf4d6f42d9827b3ce08e4)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Align release workflow with ci.yml dependencies and caching ([`f67d00b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f67d00b0ff8444832373faaa7a2dc43b879d5c18))
- Restrict release workflow trigger to specific project paths ([`d01a1cf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d01a1cfe19f54d5a65e068a6e16fe1bc067dbf79))
- Fix npm audit vulnerability ([`99e2c94`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/99e2c948f8f85b2c1adeacc61d85d7e9922bd1db))

### Refactor

- Update agent status client and server for update support ([`a69e7ad`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a69e7adf4f82ff10ee49eaebd9d2526b6f69cb18)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Use OS_DARWIN and OS_LINUX constants in install.sh ([`4db5d23`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4db5d23ae2b3412a969d01cd48846304a74acc2f)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Extract base URL into WAZUH_AGENT_REPO_URL variable in macOS update script ([`7bd57da`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7bd57dabb30ddf34d2fbd8ccc06b4e28a6e8f649))
- Replace SCRIPT_URL with WAZUH_AGENT_REPO_URL to simplify script path construction ([`7790137`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/77901379d9830a02098fe75e48953f9530ef9cff))

### Security

- Pin all actions to absolute latest stable SHAs ([`b90d3da`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b90d3da257c7844b190102a1a66bd69dd3e4d951)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Prevent script injection via repo reference variables ([`4520cf1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4520cf13e026f92c7aa03582f5874aedd87b5c4c))

## 0.4.2.rc2-user - 2026-04-14

[1a08542](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1a0854282803342fd14c5a8e85be5619e69d22f9)...[c9e1fe1](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c9e1fe1c521d1f5671c2cee7146a4e9841928c7a)

### Bug Fixes

- Update tray window positioning logic to support tauri-plugin-positioner on macos ([`391030c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/391030cf54231a1d2b57c0841517b9298e1063ee)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update window positioning logic to use BottomRight on Windows ([`d466499`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d46649958f578d9e68fb8b32f936e3b27f5f8ea4)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Ensure dock icon is hidden on macOS by setting activation policy ([`b6dadf1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b6dadf1510da77dc8c8e93eda652d4be8e940ef1)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Updated url to update script in install scripts ([`f5cc3d3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f5cc3d37e0f48788ea9869301ffaad5e9678af98))
- Correct ADORSYS_UPDATE_SCRIPT_URL to use version tags and OS-specific paths. ([`d8afe4d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d8afe4d6d7713f35e4880727ecc2771c11625ae9))
- Update ADORSYS_UPDATE_SCRIPT_URL to reference a versioned tag and use the server name variable. ([`6143ec3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6143ec3e66ae0e2c7e703250eac73176f25873fa))
- Update checksum file path to root directory in release workflow ([`b26a1c9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b26a1c96391b0db8525b5eab067db5cc6fa6195d))
- Verify binaries against release checksums, not repo ref ([`dfae694`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dfae6947e0d03746fdb9c6c87783aa49b9459687))
- Create macOS active respons bin dir ([`01c7c9c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/01c7c9c624e6e735ab79f2cf80204c74d4483596))

### Documentation

- Update README and add architecture and roadmap documentation for Wazuh agent status improvement ([`55ff861`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/55ff86100970a5eb3f98eba09e4519926c8b9632)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Restructure documentation and introduce initial ADRs for Rust migration, gRPC, and mTLS ([`8e99a8d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8e99a8da04f310503fe9c4a7e31edd78792d4c10)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update Tray UI implementation details in Rust client plan ([`e07c70a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e07c70a6e219d8da947dfc2d68fe009e9bd53f82)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>

### Features

- Initialize Wazuh Agent Status application ([`843570f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/843570f796f35f5962e3561326879d63a4158aaf)), Signed-off-by:Awambeng <awambengrodrick@gmail.com>
- Implement standalone wazuh agent status server in rust ([`0685ac4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0685ac44d3dcfe8a10236e15cfd42cecc2932dc9)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Implement modular update system and remove sudo dependency for status checks ([`b2b1069`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b2b10699cfd43be7287dce00224e177c3e05d9da)), Signed-off-by:Awambeng <awambengrodrick@gmail.com>
- Add comprehensive test suite and implement update script existence checks for Windows and Unix agents ([`ad8c618`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ad8c6188e7424fa388664052ff7cad2059321b93)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Handle uninstalled agent state by returning default values instead of errors ([`352e7d4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/352e7d4369d6f60483adef5cf0e3efe917c2ec98)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Implement real-time system metrics collection and expose via Tauri commands ([`c43c361`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c43c361d3cb30ac01d51b78f69449032713f437a)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Make server address configurable via app config and environment variables with default port 50505 ([`ff3deb0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ff3deb03583e13edb7bcc05aae30d798cd77aad2)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Add log crate and configure tauri-plugin-log with custom level filters ([`c193273`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c19327364e98c7fa6f5a49d056c30f0ee9308a4d)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update window title and improve memory usage display with new byte formatting utility ([`6f208e8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6f208e8f71446db5dadbf110d1cee62a129558b0)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Improve script robustness and error handling and update checksums, and ([`62254a8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/62254a895927d3785a69f3f69ce062a784bd78ee))

### Miscellaneous Tasks

- Add checksums.sha256 to user-main branch ([`c24f408`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c24f4081bab75805d299d00c4ff9495dd0294e5e))
- Update CHANGELOG.md and checksums ([`b7b6d2e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b7b6d2e0b51c3630eba7656d2393570719f55d61))
- Add script linting and testing workflow and update release automation triggers and paths ([`0fd0a7e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0fd0a7e00b97b8ab044bc58a7f26afea8ce86560))
- Pin GitHub Actions to specific commit SHAs for security and reproducibility ([`72372ac`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/72372ac533858926ed1d36c9e5d98b9ed3316b80))
- Address recomendations from sonarquibe ([`a14f1f1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a14f1f1397708e882b325bfed6ea463d15dce5ea))
- Add test dependency, create required directories, and move checksum generation to a dedicated job ([`20c0c6a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/20c0c6a59d791f6c13651024170e74fd4c854f4b))
- Update app version to 0.4.2.rc1 and refactor install profile initialization in installation scripts ([`08faa0e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/08faa0e2dc5be1d218222ade3a787f09ec88ec24))
- Update checksums.sha256 ([`f9cf6a3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f9cf6a33eaf3ff174cc96af271640a043aaa5d0e))
- Updated script workflow to update checksums first ([`a55add2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a55add2db6fabc92fb6661d8109f0cd0e11567d1))
- Add changelog update and checksum generation job ([`9b1d158`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9b1d158247a702d1a73aca2720eb296a92af3292))
- Update checksums.sha256 ([`cb95556`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cb95556539c625c8bedb537664ac54ba128223ad))

### Refactor

- Simplify agent status logic and remove unused configuration and update functionality ([`734d0c8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/734d0c8db8500b188cddf91d5e7ac206022d6adc)), Signed-off-by:Awambeng <awambengrodrick@gmail.com>
- Modularize frontend components and backend command handlers, and add anyhow for improved error handling ([`1f573cb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1f573cbbd29bdd393e5a1de7273d4a0e5f0cae46)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Enforce Readonly props, clean up skeleton CSS, and optimize sidebar indicator logic ([`7d2c2e0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7d2c2e0b2759a0255a541055ec749d4eab6b69b8)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Replace Math.random with cryptographically secure random values and update Google Fonts link attributes ([`c5a877b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c5a877b54999fd7d67fb39399b9ecfd6f4bb9637)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Replace window with globalThis and simplify indicatorTop calculation logic ([`590ba98`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/590ba98be3a8f4dc80c5cbaddf2f58bd3bcdf282)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Move tests to external directory, add library entry point, and implement security warning for public interface binding ([`9b6e427`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9b6e427dfa359ce222f3145aafee960313ed6051)), Signed-off-by:Awambeng <awambengrodrick@gmail.com>
- Improve agent status detection, handle missing state files, and add configurable logging and robust JSON parsing ([`dc5390e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dc5390e13697f30389869f0b06f6240201d11ef3)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Improve status provider reliability, add connection limits, and implement idle timeouts for TCP server ([`e16cf46`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e16cf464fc9d39b873fd69770f719bb0cc94cb8d)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Normalize command parsing to preserve original input in error messages and add max_connections to test configuration ([`69ef3ac`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/69ef3ac8698aed433116f5d65a4cfcc928882f5a)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Improve CLI argument handling, centralize HTTP fetching, and add Tauri capability schemas ([`9cfc542`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9cfc542a69118246371ad3d7dfd5c10e1ae179fd)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Improve agent status detection logic, optimize system metrics collection, and update tests ([`2082168`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/20821684f5e0f9b1422e2fd861f70c379c8b6cf0)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Update get_agent_status to return full AgentState instead of just AgentStatus ([`31983a4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/31983a4f006de8567190b2ed0e78e37114b24ba8))
- Remove update logic, update UI sidebar, and add tray version to agent status ([`651dfc8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/651dfc8d8b7a3ab890abf43d75902a3dc7996b75)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Remove redundant nav-item styles and improve type safety in UpdatesView component ([`3080cba`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3080cbad15c619cbe1263bbb136d1ee3a6c3c9f6)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Simplify decimal clamping and use explicit Number.parseFloat for byte formatting ([`273fefc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/273fefcde4d1c4473b6871cc4856746b51461334)), Signed-off-by:Awambeng Rodrick <awambengrodrick@gmail.com>
- Split linux and macos scripts and update readme ([`db53cef`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/db53cef26ea89e99844a2ae8741596071856d353))
- Update ADORSYS_UPDATE_SCRIPT_URL to point to the refactor branch in the wazuh-agent-status repository. ([`e4bb218`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e4bb218a71c0b0ae859dc4bc77281db2a97d0cb5))
- Parameterize repository references for update URLs ([`b38711e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b38711e923602a6da384182ba8a32b5cb860abf8))
- Centralize script logic by introducing shared utility libraries and implementing checksum verification for all scripts. ([`5e34098`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5e340988bfd336c4aceab213d92cd96c9238ff0b))
- Improve script integrity checks ([`278a78a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/278a78afe1986390b194b5a84d1a27caf1526dde))
- Standardize logging and temporary directory usage, update binary verification ([`66fd6b9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/66fd6b94b45c08294bda48339c2f058a134e6dda))
- Optimize file downloads for root users, enforce architecture compatibility across platforms, and improve checksum verification logic. ([`0c937d3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0c937d3f8ab82e359f59bbf2ca65236c1c856564))
- Implement platform-specific sed_inplace, standardize shell syntax, and improve error handling in utility scripts ([`5608090`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5608090ec26d3f20415d76786db979ac62ff0816))

## 0.4.2.rc1 - 2026-03-30

[032ca1b](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/032ca1b106c959c0b48159d84452ccd798d633b9)...[1a08542](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1a0854282803342fd14c5a8e85be5619e69d22f9)

### Bug Fixes

- Update PowerShell script URL to use the correct server version ([`465a41b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/465a41b0b57c59f82728ed436abefb32e9f3421e))

### Miscellaneous Tasks

- Update CHANGELOG.md ([`9d0b5be`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9d0b5be59b1d717cd760e98881f655ff5d846f5b))
- Add checksum generation and update changelog PR ([`87f2ff4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/87f2ff4add67683267f9755eca7a4fc31a73454f))
- Remove checksum file from release artifacts ([`1f5d641`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1f5d6410e27b78ff8dc420550302ed8b0dd72e81))
- Unify binary and script checksum generation in release workflow and add checksums file ([`1a08542`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1a0854282803342fd14c5a8e85be5619e69d22f9))

## 0.4.2-user - 2026-03-17

[da26f52](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/da26f524bc245800a465e5cbfe65c468c3c049db)...[032ca1b](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/032ca1b106c959c0b48159d84452ccd798d633b9)

### Miscellaneous Tasks

- Bump APP_VERSION  -> v0.4.2 in install scripts ([`032ca1b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/032ca1b106c959c0b48159d84452ccd798d633b9))

## 0.4.2 - 2026-03-17

[c352a2f](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c352a2f995ff41db627984a96ba20675d0eb6e8a)...[da26f52](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/da26f524bc245800a465e5cbfe65c468c3c049db)

### Bug Fixes

- Final touches for 0.4.2 release ([`da26f52`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/da26f524bc245800a465e5cbfe65c468c3c049db))

### Miscellaneous Tasks

- Bump APP_VERSION  -> v0.4.2-rc11-user ([`f621578`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f6215789fe0ac90d684a93db3ceb00a55e449dcd))

## 0.4.2-rc11-user - 2026-03-12

[2a3e02f](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2a3e02f1f9e05065ff3fc8ba3a0d24b390e4e2ed)...[c352a2f](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c352a2f995ff41db627984a96ba20675d0eb6e8a)

### Bug Fixes

- Updated group extraction logic to handle single and multiple groups correctly ([`457c78c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/457c78ccd96f127aa5d2961789bd9487f3441026))

### Features

- Add PowerShell update script support to Windows installer ([`6806294`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68062945a735c9d937fd885eb627537e090ecb75))

### Miscellaneous Tasks

- Bump APP_VERSION  -> v0.4.2-rc10-user ([`21fb049`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/21fb049e6c8b0b2b300d834568302a89a29c3b24))

### Refactor

- Split platform-specific code into separate files ([`c352a2f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c352a2f995ff41db627984a96ba20675d0eb6e8a))

## 0.4.2-rc9-user - 2026-03-11

[97f94aa](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/97f94aa28ea6493743a63c99bf952d236ba146b9)...[2a3e02f](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2a3e02f1f9e05065ff3fc8ba3a0d24b390e4e2ed)

### Bug Fixes

- Corrected adorsys update path for windows scheduled task ([`b6afdab`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b6afdab4116da644946e87844f149ecd970a7797))
- Fix: handle http fetch failures for prerelease versions ([`4d34760`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4d34760dcb468719bc8fdfecafd31e9d4f97e351))

### Miscellaneous Tasks

- Bump APP_VERSION  -> v0.4.2-rc9-user ([`2a3e02f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2a3e02f1f9e05065ff3fc8ba3a0d24b390e4e2ed))

### Refactor

- Replace adorsys-update.exe with adorsys-update.bat ([`5a03fe2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5a03fe2f34906ccfc778c402d4a484cd67494800))
- Unify error handling and platform-specific prerelease updates ([`4451493`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/44514930b8c6f299fcc3915c53c8821abfa0a093))
- Optimize log file creation for update operations ([`380b66a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/380b66ab45315935a18fcbeec3ef26855da798da))

## 0.4.2-rc8-user - 2026-03-10

[ef94d46](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ef94d46a9905839daf64e4692a49c7a3ed884cc0)...[97f94aa](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/97f94aa28ea6493743a63c99bf952d236ba146b9)

### Features

- Run Windows scheduled task as the logged-on user instead of Administrators ([`97f94aa`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/97f94aa28ea6493743a63c99bf952d236ba146b9))

## 0.4.2-rc7-user - 2026-03-10

[b3812eb](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b3812eb5a90019c00a37a511a5d0a958612a7bb4)...[ef94d46](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ef94d46a9905839daf64e4692a49c7a3ed884cc0)

### Features

- Switch Windows updater from script to binary with scheduled task and fallback methods ([`ef94d46`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ef94d46a9905839daf64e4692a49c7a3ed884cc0))

## 0.4.2-rc6-user - 2026-03-10

[aff398b](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/aff398b8de4693d4c918d4056778867b370e5ac0)...[b3812eb](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b3812eb5a90019c00a37a511a5d0a958612a7bb4)

### Features

- Feat:  refactor OS path handling ([`b3812eb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b3812eb5a90019c00a37a511a5d0a958612a7bb4))

## 0.4.2-rc5-user - 2026-03-10

[c972a11](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c972a119c25be7527a58a6661b09ae41078f33b9)...[aff398b](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/aff398b8de4693d4c918d4056778867b370e5ac0)

### Bug Fixes

- Use correct PowerShell script path for scheduled task and direct execution ([`78da22c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/78da22c1cac6b6f7720ad7fe0f5ec23acf2a1781))

### Miscellaneous Tasks

- Updated setup-agent script url in adorsys-update script ([`08d6641`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/08d6641a31c523f8677c0be7911e25a18eaaa99e))
- Bump APP_VERSION -> 0.4.2-rc5 ([`aff398b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/aff398b8de4693d4c918d4056778867b370e5ac0))

### Refactor

- Switch adorsys-update execution from .exe to .ps1 ([`318e17b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/318e17be49fba59fcfbcde5bfb677ffdc4aff95f))
- Remove scheduled task and WMI update methods ([`2d65a0b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2d65a0b9588be2383c457cebdc006e7ff92b395f))

## 0.4.2-rc1-user - 2026-03-09

[05a28b0](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/05a28b007fa91556effc175db61c6a396789ce88)...[c972a11](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c972a119c25be7527a58a6661b09ae41078f33b9)

### Refactor

- Convert updater to CLI and pass -Update flag ([`c972a11`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c972a119c25be7527a58a6661b09ae41078f33b9))

## 0.4.2-rc0-user - 2026-03-06

[9247935](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/924793521bb1e1481a548b12cd125005809347d4)...[05a28b0](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/05a28b007fa91556effc175db61c6a396789ce88)

### Bug Fixes

- Corrected file path to merge.mg ([`94951c1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/94951c1f53d558ae952a6f63cdcc0a5958ccafbc))
- Corrected windows temporary file paths ([`294ae8c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/294ae8c2c0633ef552fbc21d82f7640c21295869))
- Resolve Windows panic ([`f55acb2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f55acb271fdea52c048ec90364532014ab7ab767))
- Stdout/stderr pipe panic in windows.go ([`f2e31ca`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f2e31cacd88ee7a59803f8545b3f58929b266c8f))
- Corrected windows temporary file patten ([`00b0c46`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/00b0c465e4b58fae1e9f4ae448663f5580cf34c2))

### Features

- Implement group-based prerelease updates ([`a453df0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a453df02f13f09badb639506f86ff4db66412e2f))
- Updated app to support dynamic prerelease test groups configured upstream ([`88de391`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/88de391a03a3e60ec047752ff0cea41bc0c6d892))
- Add --no-confirm support for prerelease updates ([`070c75d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/070c75d472009d19beab7af54a34627a72a3d56d))
- Enhance prerelease update support with proper logging and UI fixes ([`0f66c90`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0f66c906c86f8609b182aa132c89385ab55fe52c))
- Enhance prerelease update handling and address further reviews from sonarquibe ([`05a28b0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/05a28b007fa91556effc175db61c6a396789ce88))

### Miscellaneous Tasks

- Reset backend ports to originals and removed built binaries ([`5bd0792`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5bd07928ce90860df3598235fb1abf0f7569995e))

### Refactor

- Address SonarQube code quality and security issues ([`50efbcf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/50efbcfac8b86b4c16c08c2333aacdb1dfc3aab4))

## 0.4.1-rc6-user - 2026-02-26

[84fb92e](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84fb92eac7e7cb25b82a3e9d478ecaa107ed119a)...[9247935](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/924793521bb1e1481a548b12cd125005809347d4)

### Miscellaneous Tasks

- Adjust polling interval to 8 hour ([`138228e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/138228e8fd1a184e7c1d1ee35299c7165d09409e))
- Added changelog and release notes generation ([`6f260ef`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6f260ef5779708af61d3af67aea0317b84011697))
- Updated ci to release binaries only ([`f639bdc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f639bdc1f85a87fa87637e366bed87c028036dbe))
- Discard release note outputs ([`9247935`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/924793521bb1e1481a548b12cd125005809347d4))

## 0.4.1-rc5-user - 2026-02-25

[d3ab3ae](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d3ab3aee39825c2e5017149197cef5369713ff2a)...[84fb92e](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84fb92eac7e7cb25b82a3e9d478ecaa107ed119a)

### Bug Fixes

- Handle GitHub release fetch errors and deduplicate backend port definition ([`622c92a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/622c92a606fc849f965c245a4b74f510c5480c51))
- Update macOS version in workflow to macos-14-large ([`ea15d74`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ea15d745676d82bf26f7ac6e1a10b8e110477990))
- Update macOS runner in workflow to arc-runner-set for improved compatibility ([`66e1acb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/66e1acb271088162b68e75ab4fb8fa7a76c0a724))

### Features

- Fetch wazuh-agent version from GitHub latest release API ([`4d28c71`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4d28c71fac85753b215cfd1a69b0b59f8e4a6505))
- Handle GitHub prerelease versions correctly in agent status ([`fbcebf9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fbcebf951d3b525e3325470a68fc4f1a0ee6ea6d))

### Miscellaneous Tasks

- Handle sonarquibe recomendations for adorsys-update.sh ([`0dbb870`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0dbb87092390f637b338a28e447dba4f077fb8d7))
- Update wazuh agent version -> 4.14.2-1 ([`f066437`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f0664376f62dfec24a8716e0846055a421c492d9))
- Rerun workflow with updated config ([`84752ba`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84752ba7b1dec0c6f8ef40073051b583cf302ade))
- Update macOS runner in workflow to macos-14-large ([`84fb92e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84fb92eac7e7cb25b82a3e9d478ecaa107ed119a))

### Ore

- Handle sonarquibe recomendations for agent status ([`c61d30b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c61d30b4b34fb8a4f5583c007a6e8d14c99b21ff))

## 0.4.1-rc4-user - 2025-12-08

[0376a60](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0376a60a7ec33ad8b871a62f0d91293a5e05f96c)...[d3ab3ae](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d3ab3aee39825c2e5017149197cef5369713ff2a)

### Bug Fixes

- Error in get agent connectivity command ([`088382a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/088382ac2b68bb6056fa30ed439f08d30ab76ff7))

### Miscellaneous Tasks

- Change agent status binary version to 0.4.0-user ([`6340575`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6340575c90492433216fc38e471d097ee8813661))
- Change wazuh agent status binary url to use WAS variable ([`b3c2c72`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b3c2c721229d40608049f1b3f04f57212960868d))
- Update binary release version to 0.4.1-rc4 ([`d3ab3ae`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d3ab3aee39825c2e5017149197cef5369713ff2a))

## 0.4.1-rc3-user - 2025-12-02

[a3f861d](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a3f861d63b922c1c319342a86b7867484df4ce0b)...[0376a60](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0376a60a7ec33ad8b871a62f0d91293a5e05f96c)

### Bug Fixes

- Use absolute paths for sudo and grep to prevent PATH injection ([`39c0d9d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/39c0d9dedaa503c6cac0f987ebe8c0984fa82638))

### Miscellaneous Tasks

- Update version in adorsys update.ps1 to latest ([`c7418fe`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c7418feda01864701ed64195468f97ae6cbce843))
- Update agent status version ([`065581b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/065581bdfc0bae52933e3479d9a50faa39662b07))
- Revert windows agent status to 0.3.4-rc3 ([`0376a60`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0376a60a7ec33ad8b871a62f0d91293a5e05f96c))

## 0.4.1-rc2-user - 2025-11-28

[3667072](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3667072caa4488e91c804c1b52832953f409c5ca)...[a3f861d](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a3f861d63b922c1c319342a86b7867484df4ce0b)

### Bug Fixes

- Update YARA command detection to use yara64 in adorsys-update.ps1 ([`1b131ed`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1b131ed4434ab56ae88a6a412acc4851eec2b0ec))
- Snort uninstall url ([`a3f861d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a3f861d63b922c1c319342a86b7867484df4ce0b))

### Features

- Add YARA uninstallation function to adorsys-update.ps1 ([`e0c5507`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e0c5507471fd5165211038658c6ae6ae3b8a4b93))

## 0.4.1-rc1-user - 2025-11-28

[f89cc6a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f89cc6a7ce60ebf8a28d36b2b7d13f2d37ab54b3)...[3667072](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3667072caa4488e91c804c1b52832953f409c5ca)

### Bug Fixes

- Fix build errors for windows ([`3667072`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3667072caa4488e91c804c1b52832953f409c5ca))

### Features

- Add installation validation checks in install.ps1 ([`deffeb7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/deffeb76cf93aaf5414e75320d8a48019fef739f))
- Add YARA installation detection and optional installation step ([`f3f1a8a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f3f1a8a909426b604649d730729e48e299730c87))

### Miscellaneous Tasks

- Update UI text for Wazuh Agent Upgrade Assistant in adorsys-update.ps1 ([`a8bf899`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a8bf8990cc66eff57051c19710c6edbaa41aba22))
- Bump App version to 0.4.1-rc1 ([`8aa31be`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8aa31be41852e6e5b7c0a324662aa77cace69557))

## 0.3.4-rc3-user - 2025-10-14

[68a628d](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68a628d973cbcf87789809088386e8c9edaf43ae)...[f89cc6a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f89cc6a7ce60ebf8a28d36b2b7d13f2d37ab54b3)

### Bug Fixes

- Update app version in install.ps1 ([`ff90abc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ff90abce7a9027aef486ca10df8387b7d603ea1d))
- Adorsys-update.ps1 only writes to active-responses.log ([`84a4c1d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84a4c1dff22312a26b23c566cbe5d1575231d0c4))
- Use scheduled task to launch adorys-update.exe ([`71c65f4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/71c65f45d1bed7d56a8b669ba37ce63f34326ac5))
- Remove time import ([`7bf378f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7bf378f170bde07cdfd01916b4d97198459db3e5))
- Ps escaping error and scheduled task xml error ([`8571a27`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8571a27a2d5f9d340b0376492af659507dd696a8))
- Set scheduled task to use administrators group ([`09e3def`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/09e3def361abaf940b11bb47e06751773669ad89))
- Replace adorsys-update.exe binary after reboot ([`24cbbd8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/24cbbd873724b5852c4af7a938ebc4e212799325))
- Install.ps1 creates run-updateswap.ps1 in order to replace binary ([`f89cc6a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f89cc6a7ce60ebf8a28d36b2b7d13f2d37ab54b3))

### Miscellaneous Tasks

- Update install.ps1 to download adorsys-update binary for windows ([`afc9605`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/afc96052c6e828a5c5d571d36336f5da567891a8))
- Launch adorsys-update binary from agent-status service ([`c7d16bd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c7d16bdf89a46c720de6917993e250649aae920b))
- Update install.ps1 to use 0.3.4-rc3 ([`95737f3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/95737f32bb1491ed921daeb58b4aa41856777860))

## 0.3.4-rc2-user - 2025-10-14

[bc2b08c](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc2b08c3a5f5a741bc530e5b92a43dbe9edefa7f)...[68a628d](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68a628d973cbcf87789809088386e8c9edaf43ae)

### Bug Fixes

- Stop service and process in install.ps1 script before downloading new one ([`97fbb0b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/97fbb0b4cf4a40b84a404a76933f3de1c47b3a0d))
- Get version correctly for adorys-update.ps1 executable ([`647adec`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/647adec8df864a5c6fe3bb654c6123614dcdd4b2))
- Set version correctly for adorys-update.ps1 executable ([`dba66ad`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dba66ad8a94ade3dfeaea3deb7ac839dae5f0d6d))
- Remove deprecated create-release action causing permission errors ([`68a628d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/68a628d973cbcf87789809088386e8c9edaf43ae))

### Features

- Add UpdateManagerAddress function to update Wazuh manager address in ossec.conf ([`bc02268`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc02268679b9f2cca8d8e87c2ee2e6ff71504a69))
- Use update installer to launch update for wazuh agent ([`a4144d9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a4144d97be9423243bfde113744b0d414047548c))

### Miscellaneous Tasks

- Add workflow to build adorsys-update binary and remove old changes to windows.go ([`a567ed8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a567ed8d8c864f5e60df19834b2840a4eca4e61b))

## 0.4.0-user - 2025-11-19

[55105a5](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/55105a51de5ae7de5aab9a5ee6e13eed470b2130)...[bc2b08c](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc2b08c3a5f5a741bc530e5b92a43dbe9edefa7f)

### Bug Fixes

- Improve defensive parsing for status and version responses ([`c1a14de`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c1a14de8c9056e70adac384a72e1cbc87e5268aa))

### Miscellaneous Tasks

- Change app version to 0.4.0 ([`ae8bd41`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ae8bd419f505dffb00a71a339c220f1e423053e2))

## 0.4.0-rc7-user - 2025-11-03

[098b89a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/098b89af6b3926d3fb00b62f689da21758d2c82a)...[55105a5](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/55105a51de5ae7de5aab9a5ee6e13eed470b2130)

### Bug Fixes

- Update agent status reporting to handle error cases consistently across platforms ([`e135707`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e135707cd9ab323fb95ad17b9b746cbb76ff5aca))

### Miscellaneous Tasks

- Update APP_VERSION -> 0.4.0-rc7 ([`55105a5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/55105a51de5ae7de5aab9a5ee6e13eed470b2130))

## 0.4.0-rc6-user - 2025-10-17

[bc5cf90](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc5cf907efd94caf60d3576bf73c9698b873404b)...[098b89a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/098b89af6b3926d3fb00b62f689da21758d2c82a)

### Bug Fixes

- Clean up imports and improve version check logic ([`e8ef796`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e8ef796a6ba1547fbf4feafc46331bdc55348dd1))

### Miscellaneous Tasks

- Update APP_VERSION to 0.4.0-rc6 in installation scripts ([`098b89a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/098b89af6b3926d3fb00b62f689da21758d2c82a))

## 0.4.0-rc5-user - 2025-10-14

[b82dae6](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b82dae69bd057a78c451033739d9c8cfb4cfd30b)...[bc5cf90](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc5cf907efd94caf60d3576bf73c9698b873404b)

### Features

- Enhance version monitoring with retry logic for valid states ([`b858629`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b858629518d6044bcb510d8a787cea7261829f8d))

### Miscellaneous Tasks

- Update version -> v0.4.0-rc5 ([`bc5cf90`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bc5cf907efd94caf60d3576bf73c9698b873404b))

## 0.4.0-rc4-user - 2025-10-09

[49cbbf1](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/49cbbf152418d1838516d328ad60485668a463ea)...[b82dae6](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b82dae69bd057a78c451033739d9c8cfb4cfd30b)

### Features

- Improve version monitoring and error handling with reduced retry intervals ([`0868dee`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0868dee972a611254f9d3b7d2c98885f1b447cc3))

### Miscellaneous Tasks

- Update APP_VERSION to 0.4.0-rc4 in install scripts ([`b82dae6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b82dae69bd057a78c451033739d9c8cfb4cfd30b))

### Refactor

- Remove low-frequency version check and allow on-demand version checks ([`8511622`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8511622990bc92832d6cac36076132dd0f86e97e))

## 0.4.0-rc3-user - 2025-10-08

[3998d2e](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3998d2e5be966966b606f764f4b1e33d13f2b5e9)...[49cbbf1](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/49cbbf152418d1838516d328ad60485668a463ea)

### Features

- Enhance version monitoring with retry logic for error states ([`071bdc7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/071bdc77e9c6cb42d51e30f64412e157e58ac65c))

### Miscellaneous Tasks

- Update APP_VERSION to 0.4.0-rc3 in install scripts ([`49cbbf1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/49cbbf152418d1838516d328ad60485668a463ea))

## 0.4.0-rc2-user - 2025-10-08

[cc3c03e](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cc3c03e70089c830789cdf05d529710beba6bb1a)...[3998d2e](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3998d2e5be966966b606f764f4b1e33d13f2b5e9)

### Miscellaneous Tasks

- Update APP_VERSION to 0.4.0-rc2 in install scripts ([`3998d2e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3998d2e5be966966b606f764f4b1e33d13f2b5e9))

### Refactor

- Remove AUTH_TOKEN from build flags for improved security and update version handling ([`b6355e9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b6355e9e6575db3d6598b29ad7f43d6c4afaf221))

## 0.4.0-rc1-user - 2025-10-08

[200fd20](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/200fd20d9742a8204a96115e7e11c2ea839b9965)...[cc3c03e](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cc3c03e70089c830789cdf05d529710beba6bb1a)

### Features

- Migrate from getlantern/systray to fyne.io/systray and update versioning logic ([`cc8af2a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cc8af2ada34e42b5d2f6a285fa1414ec91b81010))

### Miscellaneous Tasks

- Update APP_VERSION to 0.4.0-rc1 in install scripts ([`cc3c03e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cc3c03e70089c830789cdf05d529710beba6bb1a))

## 0.3.4-rc1-user - 2025-10-07

[edb4ff0](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/edb4ff0d95cc5ee451e5ed568d8e10b133be46df)...[200fd20](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/200fd20d9742a8204a96115e7e11c2ea839b9965)

### Bug Fixes

- Improve connection handling and logging in status and update functions ([`8b81ec4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8b81ec4cf48f19ec1d974402d14dade1cd023d1d))

### Features

- Improve update handling and status management across platforms from polling nature -> pub-sub behavior ([`5fdffa0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5fdffa04da54c5148078cd5ae81df2770ccdc3dd))

### Miscellaneous Tasks

- Update AUTH_TOKEN to be set at build time via ldflags for improved security ([`1b4bb6d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1b4bb6d94ac8e3b0284720ddf383ac32fc2cb150))
- Update default APP_VERSION to 0.3.4-rc1 in install scripts ([`200fd20`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/200fd20d9742a8204a96115e7e11c2ea839b9965))

## 0.3.3-user - 2025-07-16

[4f94e3d](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4f94e3d56d9b5035f711637f7803bd813144b714)...[edb4ff0](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/edb4ff0d95cc5ee451e5ed568d8e10b133be46df)

### Bug Fixes

- Integrate user-main updates ([`b3ee4d5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b3ee4d5974a6a5e69451871707848d8703bc23b7))
- Update default application version to 0.3.3 in install script ([`2e01d21`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2e01d2190bc5d3216714585ddbe0d309bed965cc))
- Add fallback for update script URL and path in install script ([`a011760`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a01176001e6243ad6075fc968a3be82936555c56))
- Update path for adorsys-update script in updateAgent function for windows ([`c4cedfa`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c4cedfa5c62d905bf59ed4565eb4cded009a1a91))
- Update script URLs to reference specific tags for consistency ([`29d00f5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/29d00f50e33db570e7755f834515ed60bb4e924b))
- Update adorsys-update script URLs to use dynamic server name for consistency ([`9b94bec`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9b94bec0d8e839c194a18d6a24538722011b833a))
- Update BASE_URL in install script to correct release path ([`48f6d6f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/48f6d6f216293ef8a809f925ef83b5333d7e7b5d))
- Unify sudo command definition across macOS and Linux files ([`f13a3fb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f13a3fb0e6042a4733eb6eabdb4e22d9087039d4))
- Correct icon path for macOS and standardize warning message function name ([`2901191`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2901191267560df34672f4f2ed14aeb2bac7cfb6))
- Add sed_alternative function for compatibility with gsed ([`54391cd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/54391cd48a8b382d5c3fa9f70332513dca52adf0))
- Rename warning_message function to warn_message for consistency ([`9f1180a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9f1180aa146a22ca2f95b91908482c620d8484f5))
- Split command arguments for grep and PowerShell commands in service status checks ([`edb4ff0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/edb4ff0d95cc5ee451e5ed568d8e10b133be46df))

### Features

- Add adorsys-update script and update installation process for windows ([`b425b86`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b425b865d30b5a1473acfa4d56894748c9022cbb))
- Add adorsys-update script download and configuration during installation in windows ([`5cd13b1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5cd13b1ad65e40f7b4a4526de4584c4dbdee658f))

## 0.3.2 - 2025-04-28

[52a2984](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/52a29845651fd9484d5c26d7ba634943cb149322)...[4f94e3d](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4f94e3d56d9b5035f711637f7803bd813144b714)

## 0.3.1 - 2025-04-14

[bd2c8f8](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bd2c8f84ebf1b9c831a67d911b4f97deb3ad9207)...[52a2984](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/52a29845651fd9484d5c26d7ba634943cb149322)

## 0.3.0 - 2025-02-24

[bdd0326](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bdd03269b82ebba19f5a42c50c9565b51cc4dd46)...[bd2c8f8](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bd2c8f84ebf1b9c831a67d911b4f97deb3ad9207)

### Bug Fixes

- Update APP_VERSION to 0.3.3 ([`593dc6a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/593dc6ab89f2d71719fbc4cfaa8594ee222840cc))
- Fix windows issues ([`261e17a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/261e17a2195fd60d76f7a37e6dc334fb3e4e96dc))
- Fix update button been enabled while update is in progress ([`905ab51`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/905ab51259ac27f87cc293d0f21dfd6c0c921bb0))
- Fix logging issue ([`54aca19`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/54aca1942fe999bc2c999f7919466589e6f0ad27))
- Add set execution policy to update agent function in order for update script to be run ([`aed4a8f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/aed4a8fac2e67f3d5aeb26122c424dd009e1a5e4))
- Change powershell path to full path to patch security vulnerability ([`08839e6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/08839e6bb2d48316c6bb4f180b5817386f60a0b3))
- Add constants to improve maintanabilty of code ([`c33a4e1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c33a4e1522fd70c40fc094c2722a6e3e22f9c9e2))
- Update version display in checkVersion functions and add comments for empty functions in darwin and linux builds ([`91e921f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/91e921f6ff4f13c9665a74bd9a3de60674508f7d))
- Improve error logging by using constants for backend connection errors ([`3fbe82e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3fbe82e225ad2581bd36cbac7066b194cc19b574))
- Change shebang from sh to bash for compatibility ([`a5e7c2d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a5e7c2d29062fd7a6aa4cc23c8544b5aa76b0faa))
- Enhance error logging for Wazuh agent status and update processes ([`53fc18a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/53fc18ae8191db107c86d922b043ea6cdfae12be))
- Fix startmonitorupdate function behavior during update and remove unused debug message in adorsys-update script ([`2f61651`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2f61651b0b9d405dd8259c70a4058a38a4d8506a))
- Fix startmonitorupdate function; streamline logging and version handling in Wazuh agent scripts and binaries ([`fb6e387`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fb6e38772900f1d54446a4b059f3f49745d083e2))
- Extend version check period to  4 hours ([`c261cf9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c261cf9a8398f218f81d221e49540a4fbb535c74))
- Add embed directive for assets in main.go ([`618a701`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/618a701bfc8098be97b4f61bc20c7c417e39ba2f))
- Add version check in monitorStatus function to handle unknown versions ([`9cf8221`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9cf8221567c29ff4cc9f5ca3cb33b1abf341f2d3))

### Features

- Feat(install): add step to download remote update script and configure it appropriately ([`6e313f0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6e313f0458f2b941378a8b4aebb78e0d407c1e24))
- Add step to remove update script ([`834d378`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/834d378952d49c3c5b21425bd3027754797ef3ac))
- Set default BASE_URL and ADORSYS_UPDATE_SCRIPT_URL if not defined ([`ad65e53`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ad65e53fc7213f15587b685522e11ce2dac4f271))
- Improve OTA update ux ([`960abb3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/960abb3f6cb1971a7c4e7d1a2b77163d865366d6))
- Enable update button only if the setup is outdated ([`59b7b8c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/59b7b8c34f2d3d6f46a21d30fd0b9dc8a3a9c26f))
- APP_VERSION -> v0.3.0 ([`056b150`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/056b1504b937a0876f9eb45354832c5555b2087f))
- Add adorsys-update.ps1 script for Wazuh agent updates and modify installation script to download and configure it ([`d02c57e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d02c57e140cd02601a1527107e5cca1325880e57))

### Miscellaneous Tasks

- Add app icon to notification on linux ([`659b020`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/659b020f50f95455e6e914214ad5d5ea98ccb525))
- Update branch to monitor online version from ([`d27d01f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d27d01f95f8ec6c54a2842e5e6f5e65c4a4d3e94))
- Update how to display version status ([`f29ff54`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f29ff54874a3a64a21dca3419b70e03fd68485e4))
- Improve logging ([`731cbc9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/731cbc99ae5984bbea10a6dec3ea2c392c3d8c9f))
- Update APP Version --> v0.3.1 ([`dd309cc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dd309cc6bab3b9ee13e40f0d8f7b1981ad22237f))
- Remove OTA update success/failure notification function ([`4bde1fa`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4bde1fa43917689e84365bd309cfa8677e968213))

### Refactor

- Reorganize environment variable definitions and validate update script existence ([`ca557ce`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ca557ce106b749449863442cd7bde2c4305528cf))
- Remove redundant comment about downloading adorsys-update.sh ([`fa523ee`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fa523ee6d175d1350bc98260d0c1c6304c7205a9))
- Remove unused pathExists function and related import ([`bde6fd1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bde6fd1317a4a48f24f472a320019944b6d6a7be))
- Streamline menu item management and enhance backend request handling ([`52c25af`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/52c25af1a1df1e00f068e9d037143ce95ac97712))
- Simplify command execution in restartAgent and updateAgent functions in linux ([`2b88101`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2b8810189b3b43714ae05cb8764d386417d8d004))
- Replace inline sudo commands with a constant in macOS and Linux files ([`128a125`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/128a12512a0e73e99cb99f992611702db6cc546a))
- Remove restartAgent function from macOS, Linux, and Windows files ([`d3d594f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d3d594fd6f7f6d4b784e851351f42dbfc524c002))

## 0.3.2-user - 2025-04-28

[c884938](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c884938908a90065d56e09bf56c9cde9387d2f29)...[bdd0326](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bdd03269b82ebba19f5a42c50c9565b51cc4dd46)

### Miscellaneous Tasks

- Remove OTA update success/failure notification function ([`bdd0326`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bdd03269b82ebba19f5a42c50c9565b51cc4dd46))

## 0.3.1-user - 2025-04-14

[1782b5c](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1782b5c15ef013f9b4eb54123f500f9ac75a4b5b)...[c884938](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c884938908a90065d56e09bf56c9cde9387d2f29)

### Bug Fixes

- Add Set-ExecutionPolicy to updateAgent function, to run upgrade script ([`39e2c9c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/39e2c9c105609a83d34201925a88b95062e16e7b))

### Miscellaneous Tasks

- APP Version --> v0.3.1 ([`035f51d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/035f51dadf999743f5decc45419235596e9f6adc))
- Update APP Version --> v0.3.1 ([`c884938`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c884938908a90065d56e09bf56c9cde9387d2f29))

## 0.3.0-user - 2025-02-24

[83c1bd7](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/83c1bd761bf0b058153993e77dca45d8b1a7963d)...[1782b5c](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1782b5c15ef013f9b4eb54123f500f9ac75a4b5b)

### Bug Fixes

- Fix logging issue ([`1782b5c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1782b5c15ef013f9b4eb54123f500f9ac75a4b5b))

### Features

- Improve ota update ux ([`101733a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/101733a52cade552ecd993040e7d1e041ba2e46b))

## 0.2.7 - 2025-02-11

[ce46afc](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ce46afc394cc9fbaf99b6da9c23683dd9bb064c4)...[83c1bd7](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/83c1bd761bf0b058153993e77dca45d8b1a7963d)

### Bug Fixes

- Unload client plist when installing new version ([`45b10d3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/45b10d3c69688ae49fdebf36468a49517309d3ff))
- Change method of checking if service exists ([`6e6c5bc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6e6c5bcd0113d21e75483f0fe36710c36f16e0db))
- Improve logging ([`a534724`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a53472445c525fcd67bfa0fe740fa316aa8c83ab))
- Stop wazuh-agent-status-client before removing ([`c382a98`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c382a98c6b2bc1f8787781d7451c4e600a035e77))
- Stop wazuh-agent-status-client before removing ([`ad7c9c4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ad7c9c461fbfdd090026796d4208bf7366a2e0e0))
- Check if wazuh-agent-status client is still running before stopping ([`ff50077`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ff50077ed94073402d039ed4b6fc1398ef5ea624))
- Change processname variable to correct Shortcutname ([`c5cdeb6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c5cdeb66ff920364afb08130a9dda8cc241165fb))

### Features

- Initial Wazuh Agent Status uninstall script ([`abc7763`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/abc7763b4ff124ba8f8d587ceceec93a8cf31a45))
- Wazuh Agent Status uninstallation script ([`225c0af`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/225c0af4413f9a8e6ff05b13e14ddbdd64106fbd))

### Miscellaneous Tasks

- Add step to unload client plist on macos ([`e9bd7ce`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e9bd7ce19c0106e1641b7c5afa2295ae31038865))
- Add step to unload client plist on macos ([`5ca31cf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5ca31cfefdb098bf710a498a516203ca6e1df012))
- Add step to unload client plist on macos ([`5839eee`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5839eee2914c7393dd00f78aea7859c44aee5d38))
- Remove bin directory ([`5bc3f78`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5bc3f78f082f73172869dcd0b36e80177cd95f93))
- Improve command to launch update script in windows ([`514a185`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/514a185e2c33ca7a393ce8482bef07b67ca9c380))
- Update app version --> 0.2.5 ([`aed60a1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/aed60a1acbf8a3dc0fef8965c6ef4f246f99d050))
- Update app version --> 0.2.6 ([`5776d7f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5776d7f9ddb4d3cd48d8abc20be108797751a04c))

### Refactor

- Improve code ([`162fc03`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/162fc0336c6d4e858b424dd59e5551a5a6f21689))

## 0.2.5-rc1 - 2025-01-23

[2b33a15](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2b33a15f4a64b4c9adbd7cd364bf8e7fcd116302)...[ce46afc](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ce46afc394cc9fbaf99b6da9c23683dd9bb064c4)

### Bug Fixes

- Remove unloadin of plist ([`60c2eb7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/60c2eb7fca751301c48f093e787e025fa8ec6ee5))
- Improve way to load plist on macos ([`6cad1d7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6cad1d79bdf2f2e2f65bceb4b1e5227d1d75b4ad))
- Improve way to load plist on macos ([`ce46afc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ce46afc394cc9fbaf99b6da9c23683dd9bb064c4))

## 0.2.5 - 2025-01-22

[4a59503](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4a595032b44c2928ac49df681186fdde325e7a27)...[2b33a15](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2b33a15f4a64b4c9adbd7cd364bf8e7fcd116302)

### Features

- APP_VERSION -> 0.2.4 ([`11bfb35`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/11bfb3568a00c5af7b496e77ed11d2d21a2d3503))

### Miscellaneous Tasks

- Improve idempotency in bash uninstall script ([`6606076`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6606076ee56389c30c64b67f23c15403c77c5668))
- Uninstall components depending on OS ([`51c7e4c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/51c7e4ca7a4ea6661ded4082c9ce354aca4c77d9))
- Uninstall components depending on OS ([`13f40da`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/13f40da6968ff2bbc13baf53eca73cf281883955))
- Remove sync feature; add restart after agent setup update ([`2b33a15`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2b33a15f4a64b4c9adbd7cd364bf8e7fcd116302))

## 0.2.4 - 2025-01-07

[cae03f1](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cae03f1e279a349bee79bde0e6b7bde87df59788)...[4a59503](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4a595032b44c2928ac49df681186fdde325e7a27)

### Features

- Update binaries download method ([`d94848e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d94848e2e59b9099e26977a14aba5a994ce56578))
- Update binaries download url ([`8d192b1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8d192b18e65891b198433b9cf5af07a455b9cdff))
- Add method to sync agent to manager ([`4a59503`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4a595032b44c2928ac49df681186fdde325e7a27))

## 0.2.7-user - 2025-02-11

[8208ccf](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8208ccf3fc134c7a6b781c53ee5ad637259b31ad)...[cae03f1](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cae03f1e279a349bee79bde0e6b7bde87df59788)

### Miscellaneous Tasks

- Sync with release v0.2.7 ([`cae03f1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cae03f1e279a349bee79bde0e6b7bde87df59788))

## 0.2.6-user - 2025-02-09

[5645033](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5645033ce9882ec7d181f637ac7327117a5bb0a9)...[8208ccf](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8208ccf3fc134c7a6b781c53ee5ad637259b31ad)

### Bug Fixes

- Improve OTA function on windows ([`8208ccf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8208ccf3fc134c7a6b781c53ee5ad637259b31ad))

### Miscellaneous Tasks

- Improve command to launch update script in windows ([`82b1b87`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/82b1b875a226494e95070c9a0ad0467875b0cf7c))

## 0.2.5-user - 2025-01-22

[53fa51a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/53fa51a1d818eb3f9c869db139b3c5113c7a2b50)...[5645033](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5645033ce9882ec7d181f637ac7327117a5bb0a9)

### Miscellaneous Tasks

- Remove sync feature ([`e8935c9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e8935c903d84b8ddfb7ec846f3b67375e5881012))
- Remove display of agent states from fetchstatus function ([`5645033`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5645033ce9882ec7d181f637ac7327117a5bb0a9))

## 0.2.4-user - 2025-01-07

[6c281d3](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6c281d33e0d7d59ff07e2b8591722090396090fd)...[53fa51a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/53fa51a1d818eb3f9c869db139b3c5113c7a2b50)

### Features

- Add method to sync agent to manager ([`79d0a8b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/79d0a8b66d2a2bbed98dca93936dfb473faae0c7))
- Add method to sync agent to manager ([`53fa51a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/53fa51a1d818eb3f9c869db139b3c5113c7a2b50))

## 0.2.3-user - 2024-12-27

[0282d3c](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0282d3c1b4e822c2c6d8e789991d62a1feb68909)...[6c281d3](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6c281d33e0d7d59ff07e2b8591722090396090fd)

### Features

- Add update feature and make windows versions work ([`da0e928`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/da0e9286f4d7d6951dcb88aebd1f04a26ff5fb21))
- Remove quit feature ([`6c281d3`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6c281d33e0d7d59ff07e2b8591722090396090fd))

### Miscellaneous Tasks

- Add log comments ([`8cb3b2e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8cb3b2e5c14f6a6a844ccf36585592bdc1e309c0))

## 0.2.3 - 2024-12-23

[ca361da](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ca361da323f3ba53b72fe2ba24c85572dbd4bd95)...[0282d3c](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0282d3c1b4e822c2c6d8e789991d62a1feb68909)

### Bug Fixes

- Put the correct download url ([`fb1573b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/fb1573b0088f53a9ebf1b8442da6cefd0f96a63a))

### Features

- Add the update functionality in the app ([`9b0ebb1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9b0ebb1000c7427ceae4b98401c824027d7bf25f))
- Change the path of the update script ([`7d6bff1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7d6bff1b03d8de29aba8be85f39e7e0126ea3833))
- Change the path of the update script ([`4898406`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4898406224e2f2dd4f32e860f7617f180cd926c8))
- APP_VERSION -> 0.2.3 ([`84912a2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/84912a2cf8a004acc0b84a375de042115e574057))

### Miscellaneous Tasks

- Add log comments ([`0282d3c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0282d3c1b4e822c2c6d8e789991d62a1feb68909))

### Refactor

- Remove unneeded packages: ([`1999475`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1999475769014dc8ce4e325b383ee5aedf971f88))

## 0.2.2 - 2024-12-20

[33f492a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/33f492aaacf061e737c235ba2a8196ae50eb16e5)...[ca361da](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ca361da323f3ba53b72fe2ba24c85572dbd4bd95)

### Bug Fixes

- Update the PROFILE switch case ([`474b227`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/474b227b0f2b9692fe0b6306c2ac91b743deae7a))
- Improve maybe_sudo in uninstall script ([`64b7c2d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/64b7c2dba3dcbfd44ef19edcbc6f07e2f89de526))
- Remove unneeded go mod tidy in windows job ([`7011c6e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7011c6e4b20aaa662dd76c566b94083a504bcc9a))
- Make windows apps bins work ([`8167ecd`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8167ecd7024a13eeabe190d1c78adc11522fb403))
- Make windows apps bins work ([`4d318a8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4d318a8cd7341252769d3f48c227509a47fcc29d))
- Make windows apps bins work ([`ca361da`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ca361da323f3ba53b72fe2ba24c85572dbd4bd95))

### Documentation

- Add steps to install app depending on profile ([`da2480a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/da2480a11ab20de0b4e09da1b81763d63ccecbb3))

### Features

- Update the install script to install binaries based on the user profile ([`5b7e0a0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5b7e0a03360dfe6687174f389eb4ddfefe48d7c1))
- Add uninstall script ([`a940707`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a940707c494e1f38611bdc4caec6fe1dafb55032))
- Add root privileges to uninstall script ([`d2d2895`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d2d2895ca85da14ffe755840f8f84fdfb9dcc064))
- Add installation validation steps ([`65b2dc7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/65b2dc7fd0638d13ce7ed117faa8651b8dd17d25))
- Build windows binaries of client and server apps ([`9dab0bf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9dab0bf792761a42198135547c1b07949d78ad82))
- Update windows install script to consider our client and server apps ([`0581acf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0581acf44aaa7a20aec514674f005b7eaf8439d1))
- Make client app run without opening terminal ([`6e1c58d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6e1c58d0069e49a05b414519d34e0f0a50cf465c))
- Make the windows install script run on most versions of powershell ([`8420dc9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8420dc93a9b7cd699e6a3936e92ee1ea739cd524))
- Make the service run on windows with admin permissions ([`69a96f0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/69a96f0c3f8a9418185472dfa932fdc019de6737))
- Make client app run without terminal on windows ([`de9f0d6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/de9f0d65db1d88743d42f23b9d5de9c9bf7d023e))

### Miscellaneous Tasks

- Improve log messages clarity ([`e8444e2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e8444e2a3928baeb58f5758beb6105b4b0496b71))

### Testing

- Make windows apps work ([`a09ec94`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a09ec942e1ab3d393fd7a8145e718bc497d8f3ad))
- Make windows apps work ([`6eab304`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6eab3041c4adac9891661d50cc8d9fa1d9a2e1ed))

## 0.2.1-user - 2024-12-03

[acf94be](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/acf94be1263e372dc9fcb4bb7215cacdff463231)...[33f492a](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/33f492aaacf061e737c235ba2a8196ae50eb16e5)

### Bug Fixes

- Remove unused function ([`563bb39`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/563bb39dddfb010d380c6df2d2e17a25da05c28d))
- Add sudo priveleges where needed ([`cd92f59`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cd92f59ac8eb1cc316e2395dd8f4fc727937b1f2))
- Add sudo priveleges where needed ([`978446c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/978446c1363b19df73719b99ebc12b31e80b9354))
- Update path of bins in macos startup units ([`97a020c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/97a020ce5acf20fe038dd8f648e4d9840cbafd11))
- Correct server listening port ([`33f492a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/33f492aaacf061e737c235ba2a8196ae50eb16e5))

### Features

- Add and integrate steps for automatic app startup on macos ([`e7d99ca`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e7d99ca99fc0759f83f2f6752a87dfbc221b4eac))

### Miscellaneous Tasks

- Remove action items and leave only status and connection states for simple users ([`11af871`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/11af871a7b287cdcaf22024b10c1ce5278caba1a))

### Refactor

- Improve code's quality ([`999d08c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/999d08c42b08a2c62b29abbb84937472c0ba5d60))
- Add display of logs ([`0d52392`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0d52392c219c9d1efc3dbbf26f4f348ff2303e8b))

## 0.2.1 - 2024-12-02

[ea39bba](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ea39bba883d761f3d67ae65221874df4d52d00e8)...[acf94be](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/acf94be1263e372dc9fcb4bb7215cacdff463231)

### Bug Fixes

- Update workflow to build amd64 ubuntu and arm64 macoss ([`5573777`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5573777ef11db56421c8c6a31d120d3766cbc109))
- Update build dist path ([`4d62bd1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4d62bd1dffc051cb47f10a023a79ccc22fd062ed))
- Update build dist path ([`8c1c81a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8c1c81a3056e8cc0af72ab06fdb08f742d551f77))
- Update build dist path ([`9669eea`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9669eeadd15e6812a569e9711b47be3cab6eefc2))
- Update build dist path ([`f754701`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f7547010dba57a3442e875dc328ceb776e3988f2))
- Improve release job ([`70708e1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/70708e18d995a8e9ca8c4c29f3bf05c9d060e26d))
- Remove uneeded brackets ([`ef679a6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ef679a651dfebb33533125a131fba45144342a25))
- Update comparison command ([`5f11598`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5f11598dae37b81df0f8493edae285bf1e3273ba))
- Update the names of the macos binaries ([`92a4f39`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/92a4f39b29e6e6d475c6b0d256330aed59b23fd9))

### Features

- Split app to client and server apps ([`81af677`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/81af677539567294378f3ca6e15923de00d9525d))
- Split app to client and server apps ([`4c16cd7`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4c16cd7a8becf65af40b986d4da5aa9bf10a7f51))
- Add service creation and desktop unit creation for ubuntu os ([`12cbb03`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/12cbb03cc60f91598c4bd8019044862c1bf2920e))

### Miscellaneous Tasks

- Add binary build for amd64 macos ([`cd37c2b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cd37c2b85de859e7920ca69576649daa024ec854))

### Refactor

- Update name of binaries ([`c42e86d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c42e86d5117a9c02422b9abacb812a815c09495d))
- Improve step execution clarity ([`0dec782`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0dec7825ddda92d375c91e2580c39c2116083125))
- Improve step execution clarity ([`28ff1fc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/28ff1fcfc05ba9c3de73d972e7f0c7ca3bf0acae))
- Improve naming in the release workflow ([`40ae311`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/40ae31180a71672a73c257c2b6c0253d76b5130f))
- Improve naming in the release workflow ([`acf94be`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/acf94be1263e372dc9fcb4bb7215cacdff463231))

## 0.1.3 - 2024-11-28

[e1cf068](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e1cf06853b0c53112836ce1ea001297e85237b4c)...[ea39bba](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ea39bba883d761f3d67ae65221874df4d52d00e8)

### Add

- Powershell script to install wazuh-agent-status app ([`b230f01`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b230f015e851a069f6822d91bbda5775792e6358))

### Bug Fixes

- Change install version ([`7ca0244`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7ca0244258f5fed9718fcbd85b6dd8f3dc008e8b))
- Update install script ([`be311d6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/be311d6438a28413c790f0acd562884dec9fb506))
- #4 make script install with portability ([`eb7a4a1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/eb7a4a14ea2c1d056a7251d789986345b911393a))
- #4 make icon path dependent on platform ([`6a24b79`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6a24b79a2bf35785a8c24fd65e120bbcac3da168))
- Remove non-needed config in service file ([`8e68f75`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8e68f750285d8007cbe4a4e3144613c8a30a3eac))
- Remove non-needed step in service file creation ([`c1b642c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c1b642ce9ccc0bd2829752352f72d0afd8251c8b))
- Delete service file if it already exists ([`8dc5139`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8dc5139dba31498d40cae0e894a6158960a18992))
- Update service config ([`6ff1d8f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6ff1d8f5d5b99916378fd1794eb7f2122a5fed12))
- Improve steps to delete service ([`5349284`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/53492848cd4a2b65e2dcdf84a4eaad6d06207a23))
- Update path to desktop unit ([`9e55b77`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9e55b77f182599541bbc185bfe19f1049918da25))
- Check if autostart folder exists ([`4208477`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/42084778184ff2a45f95c296f50a1a9bf4bdd151))
- Add step to launch app for the current session ([`f2b47b8`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f2b47b8879d14fe3cc92143df9b540cd720b0fc2))
- Remove step to launch app for the current session; add message to help use the app ([`a505fba`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a505fbad942972797b7fe619ebc89ffec0c3f74e))
- Make config to not break macos setup ([`ca3333a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ca3333ae2fd511a181b61a326591d71774941c5d))
- Run commands without sudo on linux ([`1538166`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/153816675d583e77b9bfb981b6f77f3423b3e5a1))

### Features

- Add run binary as service ([`7d17019`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7d1701937d5babf704795739c1d414db7f885b09))
- Make display access persistent across sessions and reboots ([`4b7fc21`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4b7fc2159882b64ececf61695f30bb420fd2edef))
- Grant persistent display access using .profile file ([`cd0f843`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cd0f8434026586a03799e79612a7f4d9eafbc421))
- Make agent status launch at startup using desktop unit in ubuntu ([`0819e31`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0819e31f791e8a438cb5a588301224ed9882d5f1))

### Fix

- Syntax Error line 57 ([`234b138`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/234b138c4a49dc37300674ca6f65206f55e51090))
- App Name returning True rather than wazuh-agent-status ([`50e3211`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/50e32110e7abcf5ca48bbeb418b1a47b212c1e9e))
- WOPS Version returning true instead of 0.1.2 ([`d079d4b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/d079d4b32745fc61d103549d3388f8f0fe5349b3))
- Success Message and Info Message not called correctly ([`0bcced9`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0bcced9f8aee8e6e6957c543c0f4f9dec00b1da9))

### Miscellaneous Tasks

- Copy binary to bin folder after install ([`0f74a95`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/0f74a95fc0c031fcc5db5a9e427e79113de262c5))

### Refactor

- Update how BIN_DIR is set ([`052a4fc`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/052a4fc164b346745259f611b6f76659449028f9))

## 0.1.2 - 2024-10-16

[7c90aaf](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7c90aaf60a878855ca972ba45d341a585fa4f6bb)...[e1cf068](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e1cf06853b0c53112836ce1ea001297e85237b4c)

### Bug Fixes

- Update windows build workflow ([`ea23777`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ea23777bb192cb84816fb9eaa970e2e08ab9afc1))
- Update windows build workflow ([`f343924`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f343924e455a0687f207f1bab8c9e2a947c61ae9))
- Update windows build workflow ([`018cf9e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/018cf9e9ae7cf92939db4acf4017b00bccc65e3d))
- Update windows build workflow ([`249ffe2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/249ffe29381a61c22efafe6bac79202086f00362))
- Update ci pipelines dependencies ([`58fe9df`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/58fe9df6c15f7910d2b4a6bc2b5561b20502fc6e))
- Update ci pipelines dependencies ([`2fc7e47`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2fc7e478c170686110e5986c9cdcfff4dfdc54d8))
- Update ci pipelines dependencies ([`e1cf068`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e1cf06853b0c53112836ce1ea001297e85237b4c))

### Fix

- Update monitor logic ([`bb29fac`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bb29fac01cc5418ac8fba093d97bfefbb80456cd))
- Use wazuh-control instead of systemctl ([`f9f1aff`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f9f1aff3dcc851a5b7dc945c3884ce0c3bf72592))
- Update linux status check methode ([`5ae55d0`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5ae55d07355a7c351c1e3c8bdfb49b8fadd0a0b8))
- Update status in background process every 5s ([`ea30b9e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ea30b9e3cf0a224670c65e711ac0d46815736f55))
- Split build binaries ([`b7409f4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b7409f48e9e2b104a18440048dddbb15b640aea1))
- Small fix ([`acdcf25`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/acdcf25245fe27d2a8d557981e81ead431f54edc)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>
- Remove gcc-arm-linux-gnueabi lib ([`9c6b492`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9c6b4927970681daab97f068c8437814940d9de1)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>
- Build linux amd64 and macos arm64 ([`f5a122a`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f5a122a6f90f000f261e0982b3ef03d659453980)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>
- Update README.md ([`8236cd6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8236cd622385afda9dbcc4ff240437d7aa071e0e)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>
- Update README.md ([`7dfaf6e`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7dfaf6ebbeb85c045676e347cd406e41132af299)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>
- Update build.yaml file ([`72bd093`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/72bd093a970e8fc96861963ac34b7d3b7db4f752)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>

### Miscellaneous Tasks

- Update install version to 0.1.2 ([`4134bc6`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/4134bc6647de621185ba56de68e70596b9ac911a))

### Refactoring

- Layout, logic etc.. ([`9ade877`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9ade877d3e25a1fe6df247cbdbb2efb1e6d40ad4))
- Logo layout ([`e262b57`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/e262b57e1057641d80f15579b7dde1f75fdb89d5))
- Apps printout texts ([`f81b2d2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/f81b2d2fd9d90c5a0135ca07c69d70249b444922))
- Apps printout texts ([`bac6db4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bac6db46047dfe2b478e564c8c9c4771d620c63a))
- Check status logic ([`571adc5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/571adc5b598c73e9bd08eebc7f19295aa6c764da))

### Add

- Build binary for windows ([`8d5566f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8d5566fce09509d45fa06699e3b715f6a1ea1a67)), Signed-off-by:Yannick Siewe <yannick.siewe@gmail.com>

## 0.1.1 - 2024-10-11

### Bug Fixes

- Change go version ([`17e5d02`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/17e5d02c3b974bdc96ebaef2e052a2633ecb61ed))
- Improve build pipeline ([`958642c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/958642cd28e6900f5d4f4f37f3d81f95c6a4e3d8))
- Add admin rights to command on macos ([`9acd929`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/9acd92964ef3d5944f3f9904f6c90b8ac5382b60))
- #3 applied review comments ([`5e9e365`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/5e9e365f32b7c6e677db3536be10936437f9fbf6))
- Update release workflow ([`85b2b0c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/85b2b0cd4e71c2d1f2c68a4764588ba23b4a6ccb))
- Improve macos step in workflow ([`373d1cb`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/373d1cb4eaf4c8db44bc53a02919e1491e00664c))
- Update install script ([`7c90aaf`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7c90aaf60a878855ca972ba45d341a585fa4f6bb))

### Documentation

- Add README ([`b11eff1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/b11eff1626933b4db70473e1ce48c90bdc12e7a4))
- Add README ([`2ade674`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2ade6748d613c8485f1de8b2c2a67f2c0a2deb9e))
- Update README ([`489cc1c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/489cc1ca1ac7f95a73ab8a83c4aaee392b3d5b0e))
- Update README ([`7fe032f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7fe032fec8fc09fdb04abcc2ce145f1c2c111a35))
- #4  improve README ([`260e3c5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/260e3c5b7eaa7a5c2118fc7bba2e225da4acbcef))
- #4  improve README ([`31c8a24`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/31c8a24b6ed6b73561069eb330a6158ebef5ffde))
- #4  improve README ([`6ff1232`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/6ff1232c320762782a29d6b8d0401e19fe0e4532))

### Features

- Set system tray icon based on OS ([`a252335`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a2523351cdb700fe1f9d069a70b7d1f224294a1d))

### Miscellaneous Tasks

- Add pipeline to build binaries for linux macos and windows ([`baccca1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/baccca16f1bc4da45883ef8cdb1ecea84ea424ea))
- Update workflow to generate deb and pkg packages ([`a327702`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/a327702d2949519e4661affea29889a9e3a6787f))
- Update workflow to generate deb and pkg packages ([`c9fec2f`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/c9fec2f9e0e84ea74576a5e96ccd569574018f79))
- Update workflow to generate deb and pkg packages ([`8ee45c2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8ee45c237ccb5bb2dbcef4d65677a4fadc8c9352))
- Update workflow to generate deb and pkg packages ([`ddb4ad2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/ddb4ad22381f3d9f224ae043e70b7c428da4e5b8))
- Update workflow to generate .deb file ([`2c8866c`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2c8866cfc51c5a4e71fbfb515aa97a9df0884661))
- Update workflow to upload .deb file to gh packages ([`2f9d53b`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/2f9d53b0ff2831b0128fe2d98bb0c7c5f4d2a0a9))
- Update workflow to upload .deb file to gh packages ([`bcb5a05`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bcb5a055465f71004ee4f29c204b1d0dfeab3a0f))
- Update workflow to upload .deb file to gh packages ([`bd7aa42`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/bd7aa4237a313f80f912337248067a4771e0ec0b))
- Update workflow to upload .deb file to gh packages ([`66b5986`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/66b5986770c6e2117a8da29faa5b28862fbbff20))
- Add install script ([`3d7dbc2`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/3d7dbc21d3356cacfa906c19223e59b22123e1fb))
- Add workflow to auto-generate releases ([`dab2687`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/dab2687446cfdec9dd5c3cb652b932f09058500b))
- Add workflow to auto-generate releases ([`8983749`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/8983749a09ddafae817922e326afd559cd6a3cb8))
- #3 embed logo icon in binary file and make app  buildable on MacOS ([`7a857da`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7a857dab294b32cf0889f0a998e8d287d7e181d2))
- #3 add install script ([`7f4c98d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/7f4c98dfee120288f38faff7bc43948908133321))
- #4 add workflow step to build macOS binaries ([`019f217`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/019f21740e84ccff520dbad9abefb84034e1bbae))

### Refactor

- Improve build pipeline ([`16e19e5`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/16e19e586c6171d34b884e4335a081942a13f19e))
- Improve build pipeline ([`1187409`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/1187409edd39b1f7b39453df26824e2004222a07))
- Improve build pipeline ([`18f5ab4`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/18f5ab463a75a4fde4f42f6664e66edcc2521464))
- Improve build pipeline ([`563780d`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/563780d9392eb816e3b70728940ace36f750fddb))
- Improve build pipeline ([`75c6620`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/75c6620819847c70edcf90207f3d1aee09b7da4c))
- Improve build pipeline ([`cbc3448`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/cbc344807d33db1b5da7890639373371b15ddc50))
- Improve build pipeline ([`52b65f1`](https://github.com/ADORSYS-GIS/wazuh-agent-status/commit/52b65f10b589fc3ba57d1f12c239c67c0d0b1d9d))

<!-- generated by git-cliff -->
