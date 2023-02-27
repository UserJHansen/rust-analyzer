## Getting the data

To get the data.json file you can put this js snippet in the browser, on the Mangasee website
```
var output = {}, i=0, parsedOut = [];

fetch('https://mangasee123.com/_search.php').then((r)=>r.json()).then((r)=>{
    r.map((f)=>f.i).forEach((n)=> {
        setTimeout(()=>fetch('https://mangasee123.com/manga/'+n).then(r=>r.text()).then(t=>{
            output[n] = /vm.IndexName = ([\S\s]*)vm\.Sub/.exec(t)[1]
        }), ++i*50)
    })
    setTimeout(()=>{
        for (name in output) {
            parsedOut.push({
                name,
                chapters: JSON.parse(/vm.Chapters = (.*?);\r\n\t\t\t/.exec(output[name])[1]),
                subs: parseInt(/vm.NumSubs = (.*?);/.exec(output[name])[1])
            })
        }

        console.log(parsedOut);
    },++i*50)
})
```