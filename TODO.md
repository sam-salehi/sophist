


### Decisions:
- only using gemini for now for embeddings. 


- An AI based semantic search model made in rust.
Keeps track of file movements and uses text prompts to search for files told to witness.





## General outline:
1. Make a command line tool that can add files 
- Keep track of embeddings in a simplke SQL database, maintaining address and embedding.
- Watch if the file moves or not using a Watcher.

2. Enable tokenization of files when watched.
3. Search for files.




## printing a command branch would be pretty cool too.

## commands:
1. init
2. watch
3. search
4. help
5. abandon



## files that are being supported:
// work on supporting these.
.pdf
.md
.csv


.json
.tsv
.xml


.pdf.
.doc



# Dependenceis to be added in init.


# for mac.
brew install poppler         



## life cyle of user
1. initialize
    Uses command init to pass Gemini token
    init creates SQL database
    init installls proper dependencies based on users device.
2. watch
    begin stalking different files formats given above.
    watch them for any moves and update database accordingly.
3. search
    