# cmark2jira [![Build Status](https://travis-ci.org/brandur/cmark2jira.svg?branch=master)](https://travis-ci.org/brandur/cmark2jira)

Translate good CommonMark (Markdown) into bad JIRA markup
for anyone locked into Atlassian's prison of bad software.

Functionality is very minimal. It reads CommonMark from
stdin and prints the converted content on stdout:

``` sh
cargo install
echo "*markdown!*" | cmark2jira
```

## Workflow

I have Vim setup so that I can easily pop open a new tab
containing an ephemeral file where I can write a comment
by pressing `<leader>co`:

``` vim
function! NewComment()
    let r = strftime("%Y-%m-%d_%H-%M-%S")
    execute "edit ~/Dropbox/notes/comments/blob_" . fnameescape(r) .  ".md"
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
