# dicemind

## Syntax

### Arithmetic

### Dice

### Augmentations

Augmentations are operations on the results of a dice roll before it is collapsed into a sum. They are postfix operators to the dice and consist of a letter and a number. Most of them are commutative, except for a few. They are divided into categories for convinience. 

**Filters** operate on absolute values, usually by denying the appearance of a subset of numbers.

* `d>3` - drop all values above 3.
* `k>1` - keep all values that are greater than 1, same as `d<2`.
* `d1` - drop all ones.
* `k1` - keep all ones.

A drop can always be transformed into a combination of keeps and vice versa: `k>Nk=N` is the same as `d<N`. 

**Truncations** remove dice rolls relative to the dice roll group, like highest and lowest rolls.

* `dh2` - drops the 2 highest dice.
* `kl` - keep the lowest dice (f.ex rolling with disadvantage).
* `3dNdhkh` - take the median value of a dice roll. 

The size of a dice group after a truncation is always certain.

**Misc** operations don't fit into any other category, but still are vital mechanics.

* `n6` - tally all sixes on a dice roll.
* `r1` - re-roll all ones.
* `r20x3` - re-roll any twenties 3 times.

## Examples

### D&D

`(2d6 + 2) * (2d20kh + 3 + 2 > 13)` - A level 6 Fighter (profficiency bonus +3) with +2 STR and a weapon that deals 2d6 damage attacking against with advantage a monster with 13 AC.

`2d20kh` rolling with advantage by keeping the highest dice.

### EZD6

`2d6`

### Warhammer

`6d6`

### Blades In The Dark

`3d6kh`

### Call Of Ctulhu

`d%`

### GURPS

`3d6`
 