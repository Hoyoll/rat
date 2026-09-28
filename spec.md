# Dir structure
./rat
    .LANE
    /<Lane>
        /<Task-name>
            /HEAD.md

# Lane
user given folder name just below ./rat

# Task-name
<Priority> <task-name> <HUID>

# Priority
<usize>

# HEAD.md format
    Author: <author>
    Date:   <date>

    # <task-name>
        <descriptions>

# author
get the signature and username from git

# date
using the git diff format
