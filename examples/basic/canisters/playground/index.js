import './playground.js';

// TODO get rid of this file so that we do not confuse people, just manually do the dom calls
document.write(`
    <!DOCTYPE html>

    <html>
        <head>
            <title>Sudograph Playground</title>
            <link href="https://unpkg.com/graphiql/graphiql.min.css" rel="stylesheet" />
        </head>

        <body style="margin: 0;">
            <div id="graphiql" style="height: 100vh;"></div>
        </body>
    </html>
`);