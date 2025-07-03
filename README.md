# `lt` (ell-tee)
<img width="900" alt="Screenshot 2025-07-02 at 8 48 01 PM" src="https://github.com/user-attachments/assets/105d244a-a088-47e0-b9be-387547e9b185" />

`lt` is a simple TUI application to view issues from [linear.app](https://linear.app/), for those of us still in love with the terminal. `lt` is read-only at the moment.

### Features
* View "My Issues", and the issue description, project, priority, status, tags, assignee, creator  
* Press `y` to yank (copy) the git branch name to the clipboard
* Press `o` to open the full issue in Linear desktop or web, whichever you have installed.
* **New in 0.0.4**: View switcher (`Tab`/`Shift+Tab`) - switch between custom views as defined in your Linear app
  
### Planned Features
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
![2025-07-02 20 50 09](https://github.com/user-attachments/assets/27616e50-4ac7-4cef-b6ef-88626d475ec3)
