## Getting the data

To get the data.json file you can put this js snippet in the browser, on the Mangasee website
```
var output = {},
    error_count = 0,
    i = 0,
    parsedOut = [];

fetch('https://mangasee123.com/_search.php').then((r) => r.json()).then((r) => {
    r.map((f) => f.i).forEach((n) => {
        setTimeout(() => fetch('https://mangasee123.com/manga/' + n).then(r => r.text()).then(t => {
            if (/vm.IndexName = ([\S\s]*)vm\.Sub/.exec(t) == null) {
                error_count++;
                return;
            }
            output[n] = []
            output[n][0] = /vm.IndexName = ([\S\s]*)vm\.Sub/.exec(t)[1]
            fetch('https://mangasee123.com/manga/comment.get.php', {
                method: "POST",
                body: JSON.stringify({
                    IndexName: n,
                })
            }).then(r => r.json()).then(j => output[n][1] = j.val)
        }), ++i * 35)
    })
    setTimeout(() => {
        for (name in output) {            
            parsedOut.push({
                name,
                chapters: JSON.parse(/vm.Chapters = (.*?);\r\n\t\t\t/.exec(output[name][0])[1]).map(c => ({
                    chap_no: parseInt(c.Chapter),
                    date: Math.floor(Date.parse(c.Date)/1000/60)
                })),
                subs: parseInt(/vm.NumSubs = (.*?);/.exec(output[name])[1][0]),
                comments: output[name][1].flatMap((c) => [...c.Replies.map(r => ({
                        date: Math.floor(Date.parse(r.TimeCommented)/1000/60),
                        id: parseInt(r.CommentID)
                    })),
                {
                    id: parseInt(c.CommentID),
                    date: Math.floor(Date.parse(c.TimeCommented)/1000/60)
                }])
            })
        }

        console.log(parsedOut);
    }, ++i * 35+2000)
})

```