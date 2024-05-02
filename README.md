# nepo

`nepo` is a cli tool that open files with a program specific to the file extension.

> For example, `nepo image.png` could open the image with the `viu` terminal image viewer and 
`nepo book.epub` with the `epy` epub reader.

`nepo` is configured at `~/.nepo.yml` with simple association rules

```
epubs:
  ext: 
    - epub
    - epub3
  cmd: epy ${file}
```

`nepo --mode=foo file.ext` allow you to select a different program to open the file

`nepo` is best used with shell aliases such as `function view() { nepo --mode=view "$@" }`


