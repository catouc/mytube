# FZF interactive interface shenanigans

```
mytube list-videos | fzf --delimiter '\t' --with-nth 1 --bind 'ctrl-d:execute:mytube mark-video-downloaded {2}' | cut -d $'\t' -f2
```
