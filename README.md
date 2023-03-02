## Getting the data

To get the data.json file you can put this js snippet in the browser, on the Mangasee website
```
var output = {},
    i = 0,
    parsedOut = [];

fetch('https://mangasee123.com/_search.php').then((r) => r.json()).then((r) => {
    r.map((f) => f.i).forEach((n) => {
        setTimeout(() => fetch('https://mangasee123.com/manga/' + n).then(r => r.text()).then(t => {
            output[n] = []
            output[n][0] = /vm.IndexName = ([\S\s]*)vm\.Sub/.exec(t)[1]
            fetch('https://mangasee123.com/manga/comment.get.php', {
                method: "POST",
                body: JSON.stringify({
                    IndexName: n,
                })
            }).then(r => r.json()).then(j => output[n][1] = j.val)
        }), ++i * 50)
    })
    setTimeout(() => {
        for (name in output) {
            parsedOut.push({
                name,
                chapters: JSON.parse(/vm.Chapters = (.*?);\r\n\t\t\t/.exec(output[name][0])[1]).map(c => ({
                    chap_no: parseInt(c.Chapter),
                    date: c.Date
                })),
                subs: parseInt(/vm.NumSubs = (.*?);/.exec(output[name])[1][0]),
                comments: output[name][1].map((c) => ({
                    replies: c.Replies.map(r => ({
                        date: r.TimeCommented,
                        id: parseInt(r.CommentID)
                    })),
                    id: parseInt(c.CommentID),
                    date: c.TimeCommented
                }))
            })
        }

        console.log(parsedOut);
    }, ++i * 50)
})
```