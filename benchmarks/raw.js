
const Finch = require("../");
const Handlebars = require("handlebars");
const Ejs = require("ejs");
const { performance } = require("perf_hooks"); 

Finch.addTemplate("test_1", `
<div>
{{user.name}}
{{#if user.age > 18}}
    <p>User is over 18 ({{user.age}})</p>
{{/#else}}
    <p>User is under 18 ({{user.age}})</p>
{{/}}
</div>
`);

const testhb = Handlebars.compile(`
<div>
{{user.name}}
{{#if user.isOverage}}
    <p>User is over 18</p>
{{else}}
    <p>User is under 18</p>
{{/if}}
</div>
`);

const testejs = Ejs.compile(`
<div>
<%= user.name %>
<% if (user.age > 18) { %>
    <p>User is over 18 ({{user.age}})</p>
<% } else { %>
    <p>User is under 18 ({{user.age}})</p>
<% } %>
</div>
`);


const before3 = performance.now();
for (let i=0; i < 10000; i++) {
    testejs({user: {name: "google", age: 19}});
}
console.log(`Ejs took: ${performance.now() - before3}MS`);

const before1 = performance.now();
for (let i=0; i < 10000; i++) {
    Finch.compile("test_1", {user: {name: "google", age: 19}});
}
console.log(`Finch took: ${performance.now() - before1}MS`);

const before2 = performance.now();
for (let i=0; i < 10000; i++) {
    testhb({user: {name: "google", isOverage: true}});
}
console.log(`Handlebars took: ${performance.now() - before2}MS`);