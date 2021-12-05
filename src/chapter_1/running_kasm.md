# Chapter 1: Running KASM

The Kerbal Assembler can be invoked after installation simply as `kasm`

Help can be accessed from the program itself by running:

```
kasm --help
```

The basic format for invoking `kasm` is:

```
kasm [FLAGS] [OPTIONS] <INPUT> --output <OUTPUT>
```

## Main Operation

To assemble a file of source code issue a command of the form:

```
kasm <filename> <options>
```

For example, if your code was in a file called `myfile.kasm`:

```
kasm myfile.kasm -o myfile.ko
```
This will assemble `myfile.kasm` into a KerbalObject file called `myfile.ko`

## Creating an Executable

Because KASM is just an assembler, and it produces a non-executable file, readers
of this may be wondering how to create a file they can run in kOS.

In order to that, another tool called [KLinker](https://github.com/newcomb-luke/kOS-KLinker) is required. When installing KASM on Windows
a separate "full" installer contains KLinker as well as a useful tool called [KDump](https://github.com/newcomb-luke/KDump).

If you have KLinker installed then in order to create a file that can be run in kOS you can run the following two commands:

```
kasm myfile.kasm -o myfile.ko
kld myfile.ko -o program.ksm
```

The produced `program.ksm` file can be run in kOS just like any other file.

## Options

#### The `-o` option: Specifying the output file name

KASM requires you to specify the output file path

If the output file already exists, KASM will overwrite it. KASM will by default look in the current working directory where **kasm** is run.
Although a different directory can be specified as either an absolute or relative path:

```
kasm main_program.kasm -o ~/<KSP Directory>/game.ko
```

This will produce an output file named `game.ko` in your KSP directory that you fill in.

#### The `-i` option: Specifying the include directory path

By default when the KASM preprocessor is executing [include directives]() it looks in the same directory in which the source file is.

If you are including many different files, it may be in your interests to organize those files into a directory called `include` in your project folder.

KASM allows you to still run the **kasm** command where you did before, but specify the include directory as `include`.

This can be done using the **-i** option:

MacOS/Linux: 

```
kasm fancycode.kasm -i ./include/ ...
```

Windows:

```
kasm fancycode.kasm -i .\macros\ ...
```

#### The `-f` option: Setting the file symbol name

Normally KASM outputs to the KerbalObject file format, which stores the source file's name to be referenced later
in the case of any errors. This defaults to the current input file name, but in the case of implementing an intermediary between
the user and KASM such as a compiler, a different source file name should be displayed.

In a hypothetical scenario let's say we have a compiler for a language called Yes Language where the files have `.yl` extensions: `program.yl`

This compiler produces KASM source code which is assembled using **kasm**.

This can be specified so that later error messages use the `.yl` extension using the **-f** option:

```
kasm program.kasm -f program.yl ...
```

Now errors in the linker will be reported as coming from `program.yl` so that it is less confusing for the end user.

#### The `-c` option: The executable comment

When the output KerbalObject file is linked and an executable file is created, a comment is left in the generated KSM
file that explains how the file was created. This can be read using a utility such as KDump and then the recipient of
the file can see what program compiled the code.

An example comment would currently be: `Compiled by KASM 0.11.0`

Any message can be displayed though with the `-c` option:

```
kasm program.kasm -c "KASM is pretty neat, yo" ...
```

* **Note**: The comment will only show up in the final .ksm file created by KLinker if the file has the program entry point (either `_start:` or `_init:`)

## Flags

KASM can also be passed flags that have no value following them that tell KASM what to do.

#### `-w` flag

In some cases KASM will notice something about your code that does not make it incorrect, but may not be what you want to do. These are
printed out as warnings, and in some cases you may not want to see them.

Then the **-w** flag can be used to supress warnings and only display errors:
```
kasm -w program.kasm ...
```

#### `-a` flag

KASM has a fairly capable set of preprocessor directives, but running the preprocessor may make **kasm** run slower if you
are giving it a large amount of code. If you are *not* using preprocessor directives, then you may choose to tell KASM to
not run the preprocessor at all.

This can be done by passing **kasm** the **-a** flag:
```
kasm -a program.kasm ...
```

#### `-p` flag

In contrast to the **-a** flag, the **-p** flag can be used to tell KASM that you want it to *only* run the preprocessor.
This can be useful in some cases for debugging or simply wanting all of the code to be the final code that would be in the KSM file.

```
kasm -p program.kasm -o output.kasm ...
```

## Output Format

KASM outputs in a file format called KO, or Kerbal Object file format.

This format is used by the linker to link multiple parts of a program together that were in seperate source files.

These files can be viewed using a program called [KDump](https://github.com/newcomb-luke/KDump), as well as finished KSM files.

KO files must be passed to the [linker](https://github.com/newcomb-luke/kOS-KLinker) in order to become executable by kOS.
