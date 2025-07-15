1. # JamReady - Boost Your GameJam!

   ## Introduction

   ​	JamReady was born after my friends and I participated in a GameJam  and grew frustrated with countless non-creative teamwork hurdles. We  decided to build an out-of-the-box version control system and toolkit  specifically for GameJam development.

   ## Version Control

   JamReady’s version control can be thought of as a radically simplified evolution of SVN:

   1. **Partial Local Storage:** Only files you need are stored locally, not the entire repository.
   2. **Intelligent Locking:** All files are readable. You can acquire a file's "lock" to gain exclusive write access.
   3. **Move via Metadata:** The "Move" command only updates file mapping metadata, preventing conflicts with other collaborators.
   4. **Safe Removal:** The "Remove" command only deletes the mapping. Original files can be restored via their UUID.
   5. **File Watching (In Development):** Members can "watch" files to receive notifications on changes.
   6. **Auto-Sync Binding (In Development):** Members can "bind" files to auto-sync changes to a local directory.

   ## Development Phase Management

   JamReady provides powerful tools for managing GameJam workflows:

   ### Phase Tools & Task Tracking

   1. **Phase Timer (In Development):** Set time limits for each development phase.
   2. **Flex Time (In Development:** Allocate buffer time when phases overrun.
   3. **Task/Request System (In Development):** Leads create phase tasks; members can submit cross-team requests during phases.

   ### Game Demo Updates

   1. **One-Click Distribution:** Designate a build directory. JamReady auto-compresses and uploads builds to the server.
   2. **Instant Playtesting:** Team members download and run the latest demo with a single click.
