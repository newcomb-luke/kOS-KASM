# Lists

## Table of Contents

* [List Construction](#list-construction)
* [Associated Suffixes](#associated-suffixes)
	* [Enumerable Suffixes](#enumerable-suffixes)
		* [Iterator](#iterator)
		* [Reverse Iterator](#reverse-iterator)
		* [Length](#length)
		* [Contains](#contains)
		* [Dump](#dump)
	* [List Suffixes](#list-suffixes)
		* [Add Item](#add-item)
		* [Insert Item](#insert-item)
		* [Remove Item](#remove-item)
		* [Clear List](#clear-list)
* [Element Access](#element-access)

## List Construction

Numerous built-in functions in kOS return a list.
If you wish to make your own list from scratch you can do so with the `list()`
built-in function. You pass a varying number of arguments into it to pre-populate
the list with an initial list of items.

```
push @
call #, "list()"
stol "$mylist"
```

```
push @
push 10
push 20
push 30
call #, "list()"
stol "$mylist"
```

Anything can be stored in a list, including other lists.

## Associated Suffixes

List objects are a type of `Enumerable` in kOS, and therefore those suffixes apply to Lists as well.

### Enumerable Suffixes

#### Iterator

An alternate means of iterating over an `Enumerable`. Returns an `Iterator` object.

```
push "$mylist"
gmet "iterator"
push @
call #, "<indirect>"
stol "$myiter"
```

#### Reverse Iterator

Just like Iterator, but the order of the items is reversed.

```
push "$mylist"
gmet "reverseiterator"
push @
call #, "<indirect>"
stol "$myreviter"
```

#### Length

Returns the number of elements in the enumerable as an integer.

```
push "$mylist"
gmet "length"
push @
call #, "<indirect>"
stol "$length"
```

#### Contains

Returns true if the enumerable contains an item equal to the one passed as an arguments

```
push "$mylist"
gmet "contains"
push @
push 2
call #, "<indirect>"
btr .somewhere
```

#### Empty

Returns true if the enumerable has zero items in it.

```
push "$mylist"
gmet "empty"
push @
call #, "<indirect>"
btr .somewhere
```

#### Dump

Returns a string containing a verbose dump of the enumerable’s contents.

```
push "$mylist"
gmet "empty"
push @
call #, "<indirect>"
stol "$dump"
```

### List Suffixes

#### Add Item

Appends the new value given to the end of the list.

```
push "$mylist"
gmet "add"
push @
push 2
call #, "<indirect>"
pop
```

#### Insert Item

Inserts a new value at the position given,
pushing all the other values in the list (if any) one spot to the right.

The code below inserts the value `"Hello"` into index 0.

```
push "$mylist"
gmet "insert"
push @
push 0
push "Hello"
call #, "<indirect>"
pop
```

#### Remove Item

Remove the item from the list at the numeric index given,
with counting starting at the first item being item zero

The code below removes the value at index 0.

```
push "$mylist"
gmet "remove"
push @
push 0
call #, "<indirect>"
pop
```

#### Clear List

Calling this suffix will remove all of the items currently stored in the `List`.

```
push "$mylist"
gmet "clear"
push @
call #, "<indirect>"
pop
```

#### Copy Suffix

#### Sublist

Returns a new list that contains a subset of this list starting at the given
index number, and running for the given length of items.

The code below stores a sublist of `mylist` that starts at index 1 of `mylist`
and has a length of 4.

```
push "$mylist"
gmet "sublist"
push @
push 1
push 4
call #, "<indirect>"
stol "$mysublist"
```

#### Join List into String

Returns a string created by converting each element of the array to a string,
separated by the given separator.

```
push "$mylist"
gmet "join"
push @
push ","
call #, "<indirect>"
stol "$mystring"
```

#### Find Item

Returns the first integer index within the list where an item equal to this
item can be found. Whatever the definition of “equals” is for this item type
will be used to decide if a match is found. This is a linear search from start
to finish so it can be slow if the list is long.
If no such item is found, `-1` is returned.

The code below searches for the number `5` in `mylist` and stores the result in `mylocation`.

```
push "$mylist"
gmet "find"
push @
push 5
call #, "<indirect>"
stol "$mylocation"
```

#### Index Of Item

This is just an alias for **[find](#find-item)**.

#### Find Last Item

This is the same as FIND(item), except that it searches backward instead of forward
through the list. It finds the lastmost element that is equal to the item.

The code below searches for the number `5` in `mylist` and stores the result in `mylocation`.

```
push "$mylist"
gmet "findlast"
push @
push 5
call #, "<indirect>"
stol "$mylocation"
```

#### Last Index Of Item

This is just an alias for **[findlast](#find-last-item)**.

### Element Access

In order to access individual elements in a list, we perform an operating called indexing.
All list locations start at index 0, so the first element is at index 0, and the second
at index 1, etc.

In KerboScript one uses the "list index" syntax:

```
PRINT mylist[0].
```

This prints the first element in the list.

In **kasm** one writes:

```
push "$mylist"
push 0
gidx
call #, "print()"
pop
```

This also accesses the first element in the list and prints it.
