# Internal

These are various "internal" kOS functions that usually no one really needs to use

## profileresult()

#### Takes

* Nothing

#### Description

* If you have the runtime statistics configuration option `Config:STAT` set to true, then in addition to the summary statistics
after the program run, you can also see a detailed report of the “profiling” result of your most recent program run, by calling
this function.

#### Returns

* A StringValue containing the profile data for running the script



## droppriority()

#### Takes

* Nothing

#### Description

* Drops the current trigger's running priority to the priority of the code that it interrupted

#### Returns

* Nothing (a useless 0)
