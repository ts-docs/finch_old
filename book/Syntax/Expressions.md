
# Expressions

Unlike handlebars, finch has many useful expressions so you don't have to create a custom helper just to check if two variables are equal >:(

## Literals

```
"Strings",
3.14 
true
undefined
null
```

## Path expressions

You can index variables which are objects / arrays and access their properties / elements.

**Indexing an object:**
`variable.name`

**Indexing an array:**
`variable.arr_member.0`

*(Gets the first element) ^*

## Comparions

You can use the following operators to compare two variables or literals:

```
variable1 == variable2
variable1 != variable2
variable1 > variable2
variable1 >= variable2
variable1 < variable2
variable1 <= variable2
```

## Logic

You can use `&&` (and) and `||` (or) to get a boolean result.

```
Hello {{rawName || name}}
```

## Function calls

You can call variables or path expressions (or even other function calls), and even pass parameters:

```
{{variable.fn(...parameters)}}
```
