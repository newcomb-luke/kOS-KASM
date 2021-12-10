# Math Functions

## Table of Contents

* [abs](#abs)
* [mod](#mod)
* [floor](#floor)
* [ceiling](#ceiling)
* [round](#round)
* [sqrt](#sqrt)
* [ln](#ln)
* [log10](#log10)
* [min](#min)
* [max](#max)
* [random](#random)
* [randomseed](#randomseed)
* [char](#char)
* [unchar](#unchar)
* [sin](#sin)
* [cos](#cos)
* [tan](#tan)
* [arcsin](#arcsin)
* [arccos](#arccos)
* [arctan](#arctan)
* [arctan2](#arctan2)
* [anglediff](#anglediff)

## abs()

#### Takes

* Any number type

#### Description

* Calculates the absolute value of the provided number

#### Returns

* The absolute value of the provided number (as a double)



## mod()

#### Takes 

* Any number type (divisor)
* Another argument of any number type (dividend)

#### Description

* Calculates the remainder of a division of the two numbers, also known as a modulus. (dividend % divisor)

#### Returns

* The modulus of both numbers



## floor()

#### Takes

* Any number type, the number to round
* (Optional) the number of places to round to as an integer

#### Description

* If one number is provided, it is rounded down to the next integer. 1.8 would be rounded down to 1.0
* If two numbers are provided, the first is rounded to the number of decimal places as the second. If given 1.887 and 2, the result would be 1.88

#### Returns

* The rounded value



## ceiling()

#### Takes

* Any number type, the number to round
* (Optional) the number of places to round to as an integer

#### Description

* If one number is provided, it is rounded up to the next integer. 1.1 would be rounded up to 2.0
* If two numbers are provided, the first is rounded to the number of decimal places as the second. If given 1.888 and 2, the result would be 1.89

#### Returns

* The rounded value



## round()

#### Takes

* Any number type, the number to round
* (Optional) the number of places to round as an integer

#### Description

* If one number is provided, it is rounded to to the nearest integer. 1.1 would be rounded down to 1.0, and 1.6 would be rounded up to 2.0
* If two numbers are provided, the first is rounded to the number of decimal places as the second. If given 1.888 and 2, the result would be 1.89, and
if given 1.882 and 2, the result would be 1.88

#### Returns

* The rounded value



## sqrt()

#### Takes

* Any number type, the number to take the square root of 

#### Description

* If a is the number provided, performs sqrt(a)

#### Returns

* The square root of the value



## ln()

#### Takes

* Any number type, the number to take the natural logarithm of

#### Description

* If a is the number provided, performs ln(a)

#### Returns

* The natural log of the value



## log10()

#### Takes

* Any number type, the number to take the base 10 log of

#### Description

* If a is the number provided, performs log_10(a)

#### Returns

* The log base 10 of the value



## min()

#### Takes

* 2 of any number type OR 2 of any string type

#### Description

* If numbers are provided, it returns the minimum of the two
* If strings are provided, it returns the one that is considered to be "less", useful for sorting

#### Returns

* The minimum of the two values provided



## max()

#### Takes

* 2 of any number type OR 2 of any string type

#### Description

* If numbers are provided, it returns the maximum of the two
* If strings are provided, it returns the one that is considered to be "greater", useful for sorting

#### Returns

* The maximum of the two values provided



## random()

#### Takes

* (Optional) a StringValue for the key for a named random number generator

#### Description

* If no argument is provided, then the next number from the default random number generator is provided

* If an argument is provided, you get the next number from a named random number generator. You can invent however
many keys you like and each one is a new random number generator. Supplying a key probably only means something if
you have previously used the `randomseed()` function with the same key.

#### Returns

* A pseudo-random number



## randomseed()

#### Takes

* A key to identify the initialized random number generator
* Then an integer

#### Description

* Initializes a new random number sequence from a seed, giving it a key name you can use to refer to it in future calls to `random()`

#### Returns

* Nothing (a useless 0)



## char()

#### Takes

* An integer

#### Description

* Creates a single-character string containing the unicode character specified

#### Returns

* The string



## unchar()

#### Takes

* A string

#### Description

* Converts the character in the string provided to a unicode number representing the character

#### Returns

* The integer



## sin()

#### Takes

* Any number

#### Description

* Calculates the sine of a number in degrees

#### Returns

* The result



## cos()

#### Takes

* Any number

#### Description

* Calculates the cosine of a number in degrees

#### Returns

* The result



## tan()

#### Takes

* Any number

#### Description

* Calculates the tangent of a number in degrees

#### Returns

* The result



## arcsin()

#### Takes

* Any number

#### Description

* Calculates the inverse sine of a number

#### Returns

* The result (in degrees)



## arccos()

#### Takes

* Any number

#### Description

* Calculates the inverse cosine of a number

#### Returns

* The result (in degrees)



## arctan()

#### Takes

* Any number

#### Description

* Calculates the inverse tangent of a number

#### Returns

* The result (in degrees)



## arctan2()

#### Takes

* Any number, x
* Any number, y

#### Description

* Calculates the inverse tangent of a pair of numbers as (y / x), which resolves ambiguities in the direction of the arctangent so that direction
is preserved.

#### Returns

* The result (in degrees)



## anglediff()

#### Takes

* Any number, x
* Any number, y

#### Description

* Calculates the angle that would need to be added to x, in order to get angle y. For example, calling it with 90 and 45 would return -45, because 90 - 45 =  45.

### Returns

* The difference in angle, in degrees
