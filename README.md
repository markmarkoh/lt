# `lt` (ell-tee)
<img width="900" alt="Screenshot 2025-06-08 at 4 48 29 PM" src="https://github.com/user-attachments/assets/073013c1-185f-49b9-b04a-68996a5d269c" />

`lt` is a simple TUI application to view issues from [linear.app](https://linear.app/), for those of us still in love with the terminal. `lt` is read-only at the moment.

### Features
* View "My Issues", and the issue description, project, priority, status, tags, assignee, creator  
* Press `y` to yank (copy) the git branch name to the clipboard
* Press `o` to open the full issue in Linear desktop or web, whichever you have installed.

### Planned Features
* View switcher - from custom views created in Linear to project or cycle views
* Faster loading via cacheing
* Richer markdown presentation
* Tighter local git integration
* Brew/Packager Manager installation


### Installation
**Requirements**:
* Modern terminal like kitty, Ghostty, iTerm2
* A Nerdfont installed
* A `LINEAR_API_TOKEN` environment variable
   * [Generate API token here](https://linear.app/settings/account/security)

**Cargo**  
Currently you can install `lt` using cargo:
```bash
  cargo install lt
```

Or build from source:

```bash
git clone https://github.com/markmarkoh/lt
cd lt
cargo build --release
./target/release/lt
```

## Demo 
![2025-06-08 16 50 46](https://github.com/user-attachments/assets/e05291c8-e12c-48ea-a2e2-159fee52308f)

