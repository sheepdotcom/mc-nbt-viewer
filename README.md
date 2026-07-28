# MC NBT Viewer

[![dependency status](https://deps.rs/repo/github/sheepdotcom/mc-nbt-viewer/status.svg)](https://deps.rs/repo/github/sheepdotcom/mc-nbt-viewer)
[![Build Status](https://github.com/sheepdotcom/mc-nbt-viewer/workflows/CI/badge.svg)](https://github.com/sheepdotcom/mc-nbt-viewer/actions?workflow=CI)

Hi! Welcome to my README.md file. Not sure what to really say other than this isn't done yet, and by not done I mean like as of writing this the tree ui works but looks awful and there is no file saving, all it does it read an nbt file and display it.

### How to use

You can just go [here](https://sheepdotcom.github.io/mc-nbt-viewer/) to view in your browser. If you want is as a desktop app, you gotta compile it yourself.\
Currently there is an issue with the caching of the web app, which means your browser will always keep an older version cached and won't update unless you do `ctrl` + `shift` + `r` or `ctrl` + `f5`.\
Will fix soon probably

## What needs to be done

- [ ] Fix caching problems (top priority)
- [ ] Bring back panel resizing (second)
- [ ] Embed a 3d renderer into the gui (after above maybe)
  - [ ] Render cube
  - [ ] Camera movement/rotation
  - [ ] Render textures onto cubes
  - [ ] Render textured block models
  - [ ] Block tooltip showing block info
- [ ] Download and use minecraft assets, seems like you can download them from the official site (after panel resizing maybe)
- [ ] Right click menu in the tree (low priority; don't wanna do right now)
  - [ ] adding/removing elements from a list/compound
  - [ ] renaming keys
  - [ ] finding specific keys/values in the tree
- [ ] Mod support and auto-downloading assets from modrinth/curseforge page (lowest priority)
