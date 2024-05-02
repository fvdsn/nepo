# nepo

`nepo` is a cli tool that open files with a program specific to the file extension.

For example, `nepo image.png` could open the image with the `viu` terminal image viewer and 
`nepo book.epub` with the `epy` epub reader.

`nepo` has no defaults. The association between programs and extensions is configured in a user specific 
configuration file.

`nepo --mode=mode file.ext` allow you to select a different program to open the file based on configured modes.

`nepo` is best used with shell aliases such as `function view() { nepo --mode=view "$@" }`


