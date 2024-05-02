# nepo

`nepo` is a cli tool that open files with a program depending on the file extension.

> For example, `nepo image.png` could open it with the `viu` terminal image viewer and 
`nepo book.epub` with the `epy` epub reader.



[!demo](https://github.com/fvdsn/nepo/assets/16931/6b50a25e-f8c9-474f-b1f8-a2286d87f435)



## Tldr

 - `nepo` is configured at `~/.nepo.yml` with simple association rules

    ```
    epubs:
      ext: 
        - epub
        - epub3
      cmd: epy ${file}
    ```

 - `nepo --mode=foo file.ext` allow you to select a different program to open the file

 - `nepo` is best used with shell aliases such as `function view() { nepo --mode=view "$@" }`

 - Have a look at [my personal configuration file](configs/.nepo.fvdsn.yml)

## Install

With cargo:

```
$ cargo install nepo
```

