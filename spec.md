# Dir structure
./rat
    /task
        /<Task-name>
            /HEAD.md
    /lane
        /<Lane>
            /<symlink-#Task-name>
    /tag
        /<Tag>
            /<symlink-#Task-name>
    CONFIG.toml

# Lane
user given folder name just below ./rat

# Task-name
<task-name> <HUID>

# HEAD.md format
    Author: <author>
    Date:   <date>

    # <task-name>
        <descriptions>

# author
get the signature and username from git

# date
using the git diff format
