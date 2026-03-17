# Changelog

All notable changes to this project will be documented in this file.

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
