# File I/O

## Table of Contents

* [printlist](#printlist)
* [logfile](#logfile)
* [path](#path)
* [scriptpath](#scriptpath)
* [switch](#switch)
* [cd or chdir](#cd-or-chdir)
* [copypath](#copypath)
* [movepath](#movepath)
* [deletepath](#deletepath)
* [exists](#exists)
* [open](#open)
* [create](#create)
* [createdir](#createdir)



## printlist()

#### Takes

* A string representing the type of thing to print

#### Description

* This is the KerboScript `LIST.` command. So it prints the contents of the current directory to the terminal. However there are
many different things it can list, and the default (with no arguments) version is "files". Below is a list of all possible types
to list:
	* "files"
	* "volumes"
	* "processors"
	* "bodies"
	* "targets"
	* "resources"
	* "parts"
	* "engines"
	* "rcs"
	* "sensors"
	* "config"
	* "fonts"

#### Returns

* Nothing (a useless 0)



## logfile()

#### Takes

* The file to log the data to
* The data to be logged

#### Description

* This is the KerboScript `LOG <string> TO <path>.`, so all it does is append text to a file, or it can log integers, etc as well.
* Note: `.ks` is the "default file extension" inside of kOS, so if no file extension is provided on the file path, it will add `.ks` to it 

#### Returns

* Nothing (a useless 0)



## path()

#### Takes

* A StringValue representing the path

#### Description

* Creates a `Path` structure representing the file path provided.

#### Returns

* The `Path` structure



## scriptpath()

#### Takes

* Nothing

#### Description

* Creates a `Path` structure representing the currently running file.

#### Returns

* The `Path` structure




## switch()

#### Takes

* An integer representing a VolumeId

#### Description

* The built-in `SWITCH TO <volume>.` KerboScript function, which switches the internal volume that the file system is using.

#### Returns

* Nothing (a useless 0)



## cd() or chdir()

#### Takes

* (Optional) the path to change directories to

#### Description

* Changes the directory that kOS has internally for what the current directory is. If no path is provided, it will change to the "root"
of the current volume.

#### Returns

* Nothing (a useless 0)



## copypath()

#### Takes

* A destination `Path`
* A source `Path`

#### Description

* Copies a source file to a destination. If the destination is a file, it copies the contents into a new file with that name, if the
destination is a folder, it simply copies the file into that folder.

#### Returns

* A boolean representing if the copy of multiple files was successful or not



## movepath()

#### Takes

* A destination `Path`
* A source `Path`

#### Description

* Moves a source file to a destination. If the destination is a file, it moves the contents into a new file with that name, if the
destination is a folder, it simply moves the file into that folder.

#### Returns

* A boolean representing if the movement of multiple files was successful or not



## deletepath()

#### Takes

* The path of the thing to delete

#### Description

* Deletes a file or directory on the current volume

#### Returns

* A boolean representing if the deletion was successful



## exists()

#### Takes

* A string path

#### Description

* Checks if a file or directory exists

#### Returns

* Returns true if the path exists, false if not



## open()

#### Takes

* The String path of the file or directory to open

#### Description

* Opens a file or directory if it exists

#### Returns

* If the path exists, it will return a `VolumeItem` containing a directory or file. If the path does not exist, it returns a boolean `false`



## create()

#### Takes

* The String path of the file to create

#### Description

* Creates a file with the provided path and name

#### Returns

* A `VolumeFile` containing the new file



## createdir()

#### Takes

* The String path of the directory to create

#### Description

* Creates a directory with the provided path and name

#### Returns

* A `VolumeDirectory` containing the new directory
