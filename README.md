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

 - Can iterate over multiple files or open them all at once.

 - Have a look at [my personal configuration file](configs/.nepo.fvdsn.yml)

## Install

With cargo:

```
$ cargo install nepo
```

## Configuration

It is necessary that you create a nepo configuration file called in your home
directory called `~/.nepo.yml`. This file will contain an ordered list of files assocations.

```
default:
  cmd: vim ${files}

epubs:
  ext: 
    - epub
    - epub3
  cmd: epy ${file}
  
images:
  ext: 
    - png
    - jpg
  cmd: viu ${files}
```

The name of each association is not important, but the order is.
When `nepo` opens a file, it is matched from last association to the
first. If the filename matches then the command is executed with the
provided filename. The first association is considered the default and
will always run if nothing matches.

The `cmd` configuration accepts `${file}` and `${files}` as parameters. 
The singular variant contains the first filename provided, while the plural
will contains them all.

If multiple files are provided, only the files with the matching extensions
will be provided to the command.


### Modes

If you call `nepo` with a mode, `nepo --mode=view`, It will only match assocations
with the selected mode.

```
view_json:
  mode: view
  ext: json
  cmd: jless ${file}
```

You can define any mode you want, but the `edit` and `view` mode can be selected
shorthand `nepo --view, -v, --edit, -e`
