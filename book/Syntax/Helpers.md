
# Helpers

Finch helpers act exactly like handlebar helpers, except that they can be **followed up** by another helper, which the first helper can use. 

Helpers have a name, parameters, a body, and a followup helper. Parameters, the body and the followup helper are all optional.

## Body-less helper

```
{{#template argument /}} 
```

## Helper arguments

Helpers can have an unlimited amount of arguments, which can be any expression. Arguments can be separated by either a whitespace, just like handlebars (` `), or a comma (`,`). 

## Helper body

The helper body can contain other helpers, the body of any helper must end with `{{/}}`

## Followups

You can also provide a followup helper, which will be passed to the first helper, think of it as a continuation, a **chain**. The first helper can do whatever it wants with the followup, it may render it, or it may use it's arguments or body.

The followup helper must **exactly** start right after the `/` closing of the first helper.

Example, an if-else chain:

```
{{#if value == 1}} 
    ...content...
{{/#if value == 2}}
    ...content...
{{/#else}} 
    ...content...
{{/}}
```

