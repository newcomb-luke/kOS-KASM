# General

## Table of Contents

* [print](#print)
* [printat](#printat)
* [clearscreen](#clearscreen)
* [stage](#stage)
* [toggleflybywire](#toggleflybywire)
* [selectautopilotmode](#selectautopilotmode)
* [run](#run)
* [load](#load)
* [reboot](#reboot)
* [shutdown](#shutdown)

## print()

#### Takes

* Any value to print (except ArgMarker)

#### Description

* Prints a value on the screen

#### Returns

* Nothing (a useless 0)



## printat()

#### Takes

* The row to print at
* The column to print at
* Any value to print (except ArgMarker)

#### Description

* Prints a value on the screen at the location specified

#### Returns

* Nothing (a useless 0)



## clearscreen()

#### Takes

* Nothing

#### Description

* Clears the kOS terminal screen

#### Returns

* Nothing (a useless 0)



## stage()

#### Takes

* Nothing

#### Description

* Stages the rocket, similar to hitting the spacebar

#### Returns

* Nothing (a useless 0)



## toggleflybywire()

#### Takes

* A boolean of this should enable or disable fly-by-wire
* A string representing which type of control to set

#### Description

* This is another name for "cooked control mode", where you are locking the user out of controlling the rocket. If you set the boolean to `true`
it will lock, if you set it to `false` it will unlock. The string you provide determines which set of controls it should lock out. For example
passing in `"throttle"` will lock the throttle, `"steering"` for steering, etc.

#### Returns

* Nothing (a useless 0)



## selectautopilotmode()

#### Takes

* A string

#### Description

* This essentially performs normally user-operated Navball operations. If SAS is enabled, it will set its mode. All possible SAS modes are listed below:
	* "maneuver"
	* "prograde"
	* "retrograde"
	* "normal"
	* "antinormal"
	* "radialin"
	* "radialout"
	* "target"
	* "antitarget"
	* "stability"
* An error is thrown for invalid modes

#### Returns

* Nothing (a useless 0)



## run()

#### Takes

* Arguments to itself:
	* An integer (volumeId)
	* A string path
* An (extra) ArgMarker!!
* Arguments to the program:
	* Any arguments and any number

#### Description

* The arguments to this are the volume id of the file you will be running, and the path. Then it runs the file with the arguments provided (if any)

#### Returns

* Nothing (a useless 0)




## load()

#### Takes

* A Null value, or a StringValue path of the output file of the compilation
* A boolean of whether or not to skip loading it if it has been previously loaded
* A StringValue path of the file to be loaded

#### Description

* This function is both used for compiling KerboScript files, and loading them into memory as shared libraries. Due to this, the arguments have a few
different possibilities.
* If you are trying to compile a `.ks` file:
	* The first argument should be either `"-default-compile-out-"` if you want the output file name to be the same as the input, just with a
	`.ksm` extension, or a file path.
	* The second argument should be `false`, because if the file was prevously compiled, you want to compile it again.
	* The third argument should be the path of the file you are trying to compile
* If you are trying to load a `.ks` or `.ksm` file:
	* The first argument should be Null
	* The second argument should probably be `true`, although if you want it to be run a second time if you load it twice, then it should be `false`
	* The third argument should be the path of the file you are trying to load

#### Returns

* If run in compile mode:
	* Nothing (a useless 0)
* If run in load mode:
	* A boolean stating whether the file had been previously loaded
	* An integer for where to jump to, in order to actually run the file (the entry point)




## reboot()

#### Takes

* Nothing

#### Description

* Reboots the kOS CPU this is run on

#### Returns

* Nothing, the CPU is reset




## shutdown()

#### Takes

* Nothing

#### Description

* Shuts down the kOS CPU this is run on

#### Returns

* Nothing, the CPU is shut down
