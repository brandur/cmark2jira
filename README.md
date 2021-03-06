# cmark2jira [![Build Status](https://github.com/brandur/cmark2jira/workflows/cmark2jira%20CI/badge.svg)](https://github.com/brandur/cmark2jira/actions)

Translate good CommonMark (Markdown) into bad JIRA markup.
This may be useful for anyone locked into Atlassian's
prison of bad software.

Functionality is very minimal. It reads CommonMark from
`stdin` and prints the converted content on `stdout`:

``` sh
cargo install cmark2jira
echo "*markdown!*" | cmark2jira
```

## Vim Workflow

I have Vim setup so that I can easily pop open a new tab
containing an ephemeral file where I can write a comment
by pressing `<leader>co`:

``` vim
function! NewComment()
    let r = strftime("%Y-%m-%d_%H-%M-%S")
    execute "edit ~/comments/blob_" . fnameescape(r) .  ".md"
endfunction

nnoremap <Leader>co :call NewComment()<CR>
```

After I'm done I press `<leader>ji` to run the tab's
content through `cmark2jira` and put the result into my
unnamed register (`*`) which is mapped on Mac OS to my
clipboard:

``` vim
function! ToJIRA()
    let @* = system('cmark2jira', join(getline(1,'$'), "\n"))
endfunction

nnoremap <Leader>ji :call ToJIRA()<CR>
```

I then make my way over to a browser tab with JIRA open and
paste with `Cmd+V`.

## Development

Run tests with:

``` sh
cargo test
```

### Release

To release the package:

``` sh
vi Cargo.toml # bump version number
cargo build # ensure that Cargo.lock gets updated
git add Cargo.toml Cargo.lock
git commit
cargo package
cargo publish
```
