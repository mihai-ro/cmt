# Changelog

## [2.0.7](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.6...@mihairo/cmt-v2.0.7) (2026-07-01)


### Bug Fixes

* fall back to cooked tty input when mio event source fails in hook ([#40](https://github.com/mihai-ro/cmt/issues/40)) ([b156404](https://github.com/mihai-ro/cmt/commit/b156404d8c2fdb8150178c1ac15296bb3b85dac8))

## [2.0.6](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.5...@mihairo/cmt-v2.0.6) (2026-07-01)


### Bug Fixes

* **commit:** abort cleanly when no interactive terminal available ([#37](https://github.com/mihai-ro/cmt/issues/37)) ([e191c9b](https://github.com/mihai-ro/cmt/commit/e191c9bf7eb28c327cb03af9c16b922613885d05))

## [2.0.5](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.4...@mihairo/cmt-v2.0.5) (2026-06-30)


### Bug Fixes

* picker navigation, hook stdin, line editor, quarantine ([#34](https://github.com/mihai-ro/cmt/issues/34)) ([0557beb](https://github.com/mihai-ro/cmt/commit/0557beb88e77d04a9802d1396262e083229b84ea))

## [2.0.4](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.3...@mihairo/cmt-v2.0.4) (2026-06-24)


### Bug Fixes

* wire up HomebrewFormula with correct URLs and auto-update on release ([#32](https://github.com/mihai-ro/cmt/issues/32)) ([cb8c535](https://github.com/mihai-ro/cmt/commit/cb8c5355edf904facd1850d8d2f7ccc571897cb4))

## [2.0.3](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.2...@mihairo/cmt-v2.0.3) (2026-06-24)


### Bug Fixes

* allow release-assets.githubusercontent.com in postinstall allowlist ([#30](https://github.com/mihai-ro/cmt/issues/30)) ([d966ded](https://github.com/mihai-ro/cmt/commit/d966ded2a982db88c69d9951e6e4aa19833fc97e))

## [2.0.2](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.1...@mihairo/cmt-v2.0.2) (2026-06-24)


### Bug Fixes

* correct download URL to use scoped package tag format ([#28](https://github.com/mihai-ro/cmt/issues/28)) ([983397b](https://github.com/mihai-ro/cmt/commit/983397bad1aeb9b3ad71df9ba587c8e4ce592645))

## [2.0.1](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v2.0.0...@mihairo/cmt-v2.0.1) (2026-06-24)


### Bug Fixes

* align release workflows with release-please tag format ([#26](https://github.com/mihai-ro/cmt/issues/26)) ([48cba7c](https://github.com/mihai-ro/cmt/commit/48cba7ccdee7db84f9c3456045e5b7ac8fa0d6b1))

## [2.0.0](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.3.2...@mihairo/cmt-v2.0.0) (2026-06-21)


### ⚠ BREAKING CHANGES

* rewrite cmt as a native Rust binary ([#24](https://github.com/mihai-ro/cmt/issues/24))

### Features

* rewrite cmt as a native Rust binary ([#24](https://github.com/mihai-ro/cmt/issues/24)) ([f0c9003](https://github.com/mihai-ro/cmt/commit/f0c9003f4eb7b68c612d68e91e0f60dced34f9b4))

## [1.3.2](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.3.1...@mihairo/cmt-v1.3.2) (2026-04-06)


### Bug Fixes

* replace save/restore cursor with relative cursor movement for macOS Terminal.app compatibility ([6e9e1ef](https://github.com/mihai-ro/cmt/commit/6e9e1ef0e9246e4567fe50413d54618df923b9fc))

## [1.3.1](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.3.0...@mihairo/cmt-v1.3.1) (2026-04-03)


### Bug Fixes

* separate homebrew bump from npm publish ([1737915](https://github.com/mihai-ro/cmt/commit/17379151209715741f7dc625398cdabe253e6c34))

## [1.3.0](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.2.0...@mihairo/cmt-v1.3.0) (2026-04-03)


### Features

* add _has_color_tty() for terminal detection, remove unused code ([4aa19b1](https://github.com/mihai-ro/cmt/commit/4aa19b1e4d8ec3e78931b8b2bbbfe561eba76f20))

## [1.2.0](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.6...@mihairo/cmt-v1.2.0) (2026-03-20)


### Features

* trigger homebrew-tap bump on release ([3d8c730](https://github.com/mihai-ro/cmt/commit/3d8c730290bd3a177ded1900688cac55c6d16b00))

## [1.1.6](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.5...@mihairo/cmt-v1.1.6) (2026-03-20)


### Bug Fixes

* use relative node_modules path in git hook snippets ([#17](https://github.com/mihai-ro/cmt/issues/17)) ([476499a](https://github.com/mihai-ro/cmt/commit/476499aa6f91826dbe07a254ed3571c26a06cda1))

## [1.1.5](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.4...@mihairo/cmt-v1.1.5) (2026-03-19)


### Bug Fixes

* exit 0 in hook snippets when cmt binary is not found ([#15](https://github.com/mihai-ro/cmt/issues/15)) ([cb508cd](https://github.com/mihai-ro/cmt/commit/cb508cdd883c3ff8a4dd6938889962620ff4a206))

## [1.1.4](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.3...@mihairo/cmt-v1.1.4) (2026-03-19)


### Bug Fixes

* correct release-please config and remove version field from user config ([#13](https://github.com/mihai-ro/cmt/issues/13)) ([ba5294c](https://github.com/mihai-ro/cmt/commit/ba5294cde7e837b2e3a955a97d54ab21288b528d))

## [1.1.3](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.2...@mihairo/cmt-v1.1.3) (2026-03-19)


### Bug Fixes

* exit early when no staged changes instead of prompting to continue ([#11](https://github.com/mihai-ro/cmt/issues/11)) ([cd707e0](https://github.com/mihai-ro/cmt/commit/cd707e0d9ec6bf67342aac14e0dcdd13a07443de))

## [1.1.2](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.1...@mihairo/cmt-v1.1.2) (2026-03-18)


### Bug Fixes

* picker and default commit ([#9](https://github.com/mihai-ro/cmt/issues/9)) ([e92831b](https://github.com/mihai-ro/cmt/commit/e92831b724a60c8e06a5f123d7cd2f865c9ad779))

## [1.1.1](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.1.0...@mihairo/cmt-v1.1.1) (2026-03-18)


### Bug Fixes

* UI polish and release config ([#7](https://github.com/mihai-ro/cmt/issues/7)) ([bf6abb7](https://github.com/mihai-ro/cmt/commit/bf6abb7b97bb9801c11d3b45f724a34b92d7059b))

## [1.1.0](https://github.com/mihai-ro/cmt/compare/@mihairo/cmt-v1.0.1...@mihairo/cmt-v1.1.0) (2026-03-18)


### Features

* initial release v1.0.0 ([c24be64](https://github.com/mihai-ro/cmt/commit/c24be649d4133b4438adc68979af01c887ec77d8))


### Bug Fixes

* build commit preview box dynamically using content max width ([b7c464b](https://github.com/mihai-ro/cmt/commit/b7c464b646df0fd94cd74c713399edda12518635))
* build commit preview box dynamically using content max width ([ee5724a](https://github.com/mihai-ro/cmt/commit/ee5724a412c6cbd681a09817c09507f3a586e050))

## [1.0.1](https://github.com/mihai-ro/cmt/compare/v1.0.0...v1.0.1) (2026-03-18)


### Bug Fixes

* build commit preview box dynamically using content max width ([b7c464b](https://github.com/mihai-ro/cmt/commit/b7c464b646df0fd94cd74c713399edda12518635))
* build commit preview box dynamically using content max width ([ee5724a](https://github.com/mihai-ro/cmt/commit/ee5724a412c6cbd681a09817c09507f3a586e050))
