# Source Layout
Having never worked with typescript before, I find it helpful to describe exactly where things go.

 - `name-ext/`
    - `bin/`: contains compiled binaries invoked by commands
    - `src/`: contains all the extension source code
        - `commands/`: source code for contributed commands found in the command palette
        - `helpers/`: source code for helper functions which do not necessarily contribute anything on their own
        - `test/`
    - `extension.ts`: main
    - `layout.md`: you are here
    - `tree.ts`: data structure definition for a dummy tree used to create the sidebar
    - `package.json`: definition of contribution points