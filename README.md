# `lt` (ell-tee)
<img width="900" alt="Basic intro page for lt" src="https://github.com/user-attachments/assets/dd29d164-4ec8-4bb6-b469-667680b2d739" />
<img width="900" alt="Search results activated" src="https://github.com/user-attachments/assets/62d426a3-fe34-4eb2-a44a-0e53da1e03e9" />

`lt` is a simple TUI application to view issues from [linear.app](https://linear.app/), for those of us still in love with the terminal. `lt` is read-only at the moment.

### Features
* View "My Issues", and the issue description, project, priority, status, tags, assignee, creator  
* Press `y` to yank (copy) the git branch name to the clipboard
* Press `o` to open the full issue in Linear desktop or web, whichever you have installed.
* **New in 0.0.4**: View switcher (`Tab`/`Shift+Tab`) - switch between custom views as defined in your Linear app
* **New in 0.0.6**: Now available to install view Homebrew (see **Installation**)
* **New in 0.0.7**: Search issues (`/`) - search all issues by simple search term
  
### Planned Features
* Faster loading via cacheing
* Richer markdown presentation
* Brew/Packager Manager installation improvements


### Installation
**Requirements**:
* Modern terminal like kitty, Ghostty, iTerm2
* A Nerdfont installed
* A `LINEAR_API_TOKEN` environment variable
   * [Generate API token here](https://linear.app/settings/account/security)

**Homebrew (Mac)**
```bash
brew tap markmarkoh/lt
brew install lt
```

**Cargo**  
You can install `lt` using cargo on any OS:
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
![2025-07-12 10 35 25](https://github.com/user-attachments/assets/34460f44-ee91-416d-8acf-4c7b3a4d7b75)
