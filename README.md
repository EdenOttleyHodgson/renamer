# Renamer
Tool for renaming files using user specified patterns

## Patterns
Patterns are made up of two parts: capture groups and the pattern itself, 
separated by a `|`.   

Capture groups are pieces of the original filename that can be used in the pattern.
They are specifed with a number, followed by a regular expression enclosed in quotation marks.
Example:
```
1"[a-z]*"2"[A-Z]*"
```
This matches every lowercase character and assigns it to capture group 1, while
every uppercase character is matched to capture group 2.

The pattern itself is then composed of literals and inserts. Literals can be any 
valid character and are directly inserted into the new file name.

Inserts are special directives for including dynamic information in the new filename
that are enclosed by forward slashes(`/`). 

Example:
```
pic/RAND/
```
Will rename every file applied to pic followed by a random 32 bit integer.

### Insert List
| Insert            | Description                                       |
| ------------------| --------------------------------------------------|
| /RAND/            | A random 32 bit integer.                          |
| /ORIGINAL/        | The original text of the file.                    |
| /capX/            | The text of the capture group specified by X.     |
| /DATE_MODIFIED/   | The date the file was last modified.              |
| /NOW/             | The current date.                                 | 

