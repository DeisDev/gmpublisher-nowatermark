# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- On first launch, nwmpublisher detects the settings left behind by an older gmpublisher install
  and offers to import them (destinations, local addon paths and preferences). Declining writes out
  fresh settings so the offer isn't repeated.

### Changed

- **Renamed the app to `nwmpublisher`.** This covers the executable and installer, the window title,
  the CLI name, the `.gma` file association and MSI registry entries, the config, temp and log
  folders, and the WebSocket subprotocol.
- The update checker and the GitHub links now point at
  [DeisDev/nwmpublisher](https://github.com/DeisDev/nwmpublisher) instead of upstream gmpublisher.
- Credits now name DeisDev "CatSniffer" as the maintainer. The MSI publisher and Cargo authors list
  include DeisDev alongside William Venner, whose attribution is kept.
- Published GMA files are no longer always named `gmpublisher.gma`. The file name now follows the
  addon title entered in the publish window, and can be overridden with any name you like.
- If no name is given, the packed file is named `publishedaddon.gma`.
- The default Workshop preview icon is now your Steam avatar with a white outline around it, instead
  of the gmpublisher logo.

### Removed

- Newly published Workshop items no longer get "Uploaded with gmpublisher" written into their
  description.
- The Steam status shown while the app is running no longer reads "In gmpublisher"; it now reads
  "In the Workshop".