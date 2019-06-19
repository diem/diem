Release Process
====================

## Branch updates

### Before every release candidate

* Update translations (ping wumpus on IRC) see [translation_process.md](https://github.com/bitcoin/bitcoin/blob/master/doc/translation_process.md#synchronising-translations).
* Update manpages, see [gen-manpages.sh](https://github.com/bitcoin/bitcoin/blob/master/contrib/devtools/README.md#gen-manpagessh).
* Update release candidate version in `configure.ac` (`CLIENT_VERSION_RC`).

### Before every major and minor release

* Update [bips.md](bips.md) to account for changes since the last release.
* Update version in `configure.ac` (don't forget to set `CLIENT_VERSION_RC` to `0`).
* Write release notes (see "Write the release notes" below).
* Update `src/chainparams.cpp` nMinimumChainWork with information from the getblockchaininfo rpc.
* Update `src/chainparams.cpp` defaultAssumeValid with information from the getblockhash rpc.
  - The selected value must not be orphaned so it may be useful to set the value two blocks back from the tip.
  - Testnet should be set some tens of thousands back from the tip due to reorgs there.
  - This update should be reviewed with a reindex-chainstate with assumevalid=0 to catch any defect
     that causes rejection of blocks in the past history.

### Before every major release

* Update hardcoded [seeds](/contrib/seeds/README.md), see [this pull request](https://github.com/bitcoin/bitcoin/pull/7415) for an example.
* Update [`src/chainparams.cpp`](/src/chainparams.cpp) m_assumed_blockchain_size and m_assumed_chain_state_size with the current size plus some overhead.
* Update `src/chainparams.cpp` chainTxData with statistics about the transaction count and rate. Use the output of the RPC `getchaintxstats`, see
  [this pull request](https://github.com/bitcoin/bitcoin/pull/12270) for an example. Reviewers can verify the results by running `getchaintxstats <window_block_count> <window_last_block_hash>` with the `window_block_count` and `window_last_block_hash` from your output.
* On both the master branch and the new release branch:
  - update `CLIENT_VERSION_MINOR` in [`configure.ac`](../configure.ac)
  - update `CLIENT_VERSION_MINOR`, `PACKAGE_VERSION`, and `PACKAGE_STRING` in [`build_msvc/bitcoin_config.h`](/build_msvc/bitcoin_config.h)
* On the new release branch in [`configure.ac`](../configure.ac) and [`build_msvc/bitcoin_config.h`](/build_msvc/bitcoin_config.h) (see [this commit](https://github.com/bitcoin/bitcoin/commit/742f7dd)):
  - set `CLIENT_VERSION_REVISION` to `0`
  - set `CLIENT_VERSION_IS_RELEASE` to `true`

#### Before branch-off

- Clear the release notes and move them to the wiki (see "Write the release notes" below).

#### After branch-off (on master)

- Update the version of `contrib/gitian-descriptors/*.yml`.

#### After branch-off (on the major release branch)

- Update the versions and the link to the release notes draft in `doc/release-notes.md`.

#### Before final release

- Merge the release notes from the wiki into the branch.
- Ensure the "Needs release note" label is removed from all relevant pull requests and issues.


## Building

### First time / New builders

If you're using the automated script (found in [contrib/gitian-build.py](/contrib/gitian-build.py)), then at this point you should run it with the "--setup" command. Otherwise ignore this.

Check out the source code in the following directory hierarchy.

    cd /path/to/your/toplevel/build
    git clone https://github.com/bitcoin-core/gitian.sigs.git
    git clone https://github.com/bitcoin-core/bitcoin-detached-sigs.git
    git clone https://github.com/devrandom/gitian-builder.git
    git clone https://github.com/bitcoin/bitcoin.git

### Write the release notes

Open a draft of the release notes for collaborative editing at https://github.com/bitcoin-core/bitcoin-devwiki/wiki.

For the period during which the notes are being edited on the wiki, the version on the branch should be wiped and replaced with a link to the wiki which should be used for all announcements until `-final`.

Write the release notes. `git shortlog` helps a lot, for example:

    git shortlog --no-merges v(current version, e.g. 0.19.2)..v(new version, e.g. 0.20.0)

(or ping @wumpus on IRC, he has specific tooling to generate the list of merged pulls
and sort them into categories based on labels).

Generate list of authors:

    git log --format='- %aN' v(current version, e.g. 0.20.0)..v(new version, e.g. 0.20.1) | sort -fiu

Tag the version (or release candidate) in git:

    git tag -s v(new version, e.g. 0.20.0)

### Setup and perform Gitian builds

If you're using the automated script (found in [contrib/gitian-build.py](/contrib/gitian-build.py)), then at this point you should run it with the "--build" command. Otherwise ignore this.

Setup Gitian descriptors:

    pushd ./bitcoin
    export SIGNER="(your Gitian key, ie bluematt, sipa, etc)"
    export VERSION=(new version, e.g. 0.20.0)
    git fetch
    git checkout v${VERSION}
    popd

Ensure your gitian.sigs are up-to-date if you wish to gverify your builds against other Gitian signatures.

    pushd ./gitian.sigs
    git pull
    popd

Ensure gitian-builder is up-to-date:

    pushd ./gitian-builder
    git pull
    popd

### Fetch and create inputs: (first time, or when dependency versions change)

    pushd ./gitian-builder
    mkdir -p inputs
    wget -P inputs https://bitcoincore.org/cfields/osslsigncode-Backports-to-1.7.1.patch
    echo 'a8c4e9cafba922f89de0df1f2152e7be286aba73f78505169bc351a7938dd911 inputs/osslsigncode-Backports-to-1.7.1.patch' | sha256sum -c
    wget -P inputs https://downloads.sourceforge.net/project/osslsigncode/osslsigncode/osslsigncode-1.7.1.tar.gz
    echo 'f9a8cdb38b9c309326764ebc937cba1523a3a751a7ab05df3ecc99d18ae466c9 inputs/osslsigncode-1.7.1.tar.gz' | sha256sum -c
    popd

Create the macOS SDK tarball, see the [macOS build instructions](build-osx.md#deterministic-macos-dmg-notes) for details, and copy it into the inputs directory.

### Optional: Seed the Gitian sources cache and offline git repositories

NOTE: Gitian is sometimes unable to download files. If you have errors, try the step below.

By default, Gitian will fetch source files as needed. To cache them ahead of time, make sure you have checked out the tag you want to build in bitcoin, then:

    pushd ./gitian-builder
    make -C ../bitcoin/depends download SOURCES_PATH=`pwd`/cache/common
    popd

Only missing files will be fetched, so this is safe to re-run for each build.

NOTE: Offline builds must use the --url flag to ensure Gitian fetches only from local URLs. For example:

    pushd ./gitian-builder
    ./bin/gbuild --url bitcoin=/path/to/bitcoin,signature=/path/to/sigs {rest of arguments}
    popd

The gbuild invocations below <b>DO NOT DO THIS</b> by default.

### Build and sign Bitcoin Core for Linux, Windows, and macOS:

    pushd ./gitian-builder
    ./bin/gbuild --num-make 2 --memory 3000 --commit bitcoin=v${VERSION} ../bitcoin/contrib/gitian-descriptors/gitian-linux.yml
    ./bin/gsign --signer "$SIGNER" --release ${VERSION}-linux --destination ../gitian.sigs/ ../bitcoin/contrib/gitian-descriptors/gitian-linux.yml
    mv build/out/bitcoin-*.tar.gz build/out/src/bitcoin-*.tar.gz ../

    ./bin/gbuild --num-make 2 --memory 3000 --commit bitcoin=v${VERSION} ../bitcoin/contrib/gitian-descriptors/gitian-win.yml
    ./bin/gsign --signer "$SIGNER" --release ${VERSION}-win-unsigned --destination ../gitian.sigs/ ../bitcoin/contrib/gitian-descriptors/gitian-win.yml
    mv build/out/bitcoin-*-win-unsigned.tar.gz inputs/bitcoin-win-unsigned.tar.gz
    mv build/out/bitcoin-*.zip build/out/bitcoin-*.exe ../

    ./bin/gbuild --num-make 2 --memory 3000 --commit bitcoin=v${VERSION} ../bitcoin/contrib/gitian-descriptors/gitian-osx.yml
    ./bin/gsign --signer "$SIGNER" --release ${VERSION}-osx-unsigned --destination ../gitian.sigs/ ../bitcoin/contrib/gitian-descriptors/gitian-osx.yml
    mv build/out/bitcoin-*-osx-unsigned.tar.gz inputs/bitcoin-osx-unsigned.tar.gz
    mv build/out/bitcoin-*.tar.gz build/out/bitcoin-*.dmg ../
    popd

Build output expected:

  1. source tarball (`bitcoin-${VERSION}.tar.gz`)
  2. linux 32-bit and 64-bit dist tarballs (`bitcoin-${VERSION}-linux[32|64].tar.gz`)
  3. windows 32-bit and 64-bit unsigned installers and dist zips (`bitcoin-${VERSION}-win[32|64]-setup-unsigned.exe`, `bitcoin-${VERSION}-win[32|64].zip`)
  4. macOS unsigned installer and dist tarball (`bitcoin-${VERSION}-osx-unsigned.dmg`, `bitcoin-${VERSION}-osx64.tar.gz`)
  5. Gitian signatures (in `gitian.sigs/${VERSION}-<linux|{win,osx}-unsigned>/(your Gitian key)/`)

### Verify other gitian builders signatures to your own. (Optional)

Add other gitian builders keys to your gpg keyring, and/or refresh keys: See `../bitcoin/contrib/gitian-keys/README.md`.

Verify the signatures

    pushd ./gitian-builder
    ./bin/gverify -v -d ../gitian.sigs/ -r ${VERSION}-linux ../bitcoin/contrib/gitian-descriptors/gitian-linux.yml
    ./bin/gverify -v -d ../gitian.sigs/ -r ${VERSION}-win-unsigned ../bitcoin/contrib/gitian-descriptors/gitian-win.yml
    ./bin/gverify -v -d ../gitian.sigs/ -r ${VERSION}-osx-unsigned ../bitcoin/contrib/gitian-descriptors/gitian-osx.yml
    popd

### Next steps:

Commit your signature to gitian.sigs:

    pushd gitian.sigs
    git add ${VERSION}-linux/"${SIGNER}"
    git add ${VERSION}-win-unsigned/"${SIGNER}"
    git add ${VERSION}-osx-unsigned/"${SIGNER}"
    git commit -m "Add ${VERSION} unsigned sigs for ${SIGNER}"
    git push  # Assuming you can push to the gitian.sigs tree
    popd

Codesigner only: Create Windows/macOS detached signatures:
- Only one person handles codesigning. Everyone else should skip to the next step.
- Only once the Windows/macOS builds each have 3 matching signatures may they be signed with their respective release keys.

Codesigner only: Sign the macOS binary:

    transfer bitcoin-osx-unsigned.tar.gz to macOS for signing
    tar xf bitcoin-osx-unsigned.tar.gz
    ./detached-sig-create.sh -s "Key ID"
    Enter the keychain password and authorize the signature
    Move signature-osx.tar.gz back to the gitian host

Codesigner only: Sign the windows binaries:

    tar xf bitcoin-win-unsigned.tar.gz
    ./detached-sig-create.sh -key /path/to/codesign.key
    Enter the passphrase for the key when prompted
    signature-win.tar.gz will be created

Codesigner only: Commit the detached codesign payloads:

    cd ~/bitcoin-detached-sigs
    checkout the appropriate branch for this release series
    rm -rf *
    tar xf signature-osx.tar.gz
    tar xf signature-win.tar.gz
    git add -a
    git commit -m "point to ${VERSION}"
    git tag -s v${VERSION} HEAD
    git push the current branch and new tag

Non-codesigners: wait for Windows/macOS detached signatures:

- Once the Windows/macOS builds each have 3 matching signatures, they will be signed with their respective release keys.
- Detached signatures will then be committed to the [bitcoin-detached-sigs](https://github.com/bitcoin-core/bitcoin-detached-sigs) repository, which can be combined with the unsigned apps to create signed binaries.

Create (and optionally verify) the signed macOS binary:

    pushd ./gitian-builder
    ./bin/gbuild -i --commit signature=v${VERSION} ../bitcoin/contrib/gitian-descriptors/gitian-osx-signer.yml
    ./bin/gsign --signer "$SIGNER" --release ${VERSION}-osx-signed --destination ../gitian.sigs/ ../bitcoin/contrib/gitian-descriptors/gitian-osx-signer.yml
    ./bin/gverify -v -d ../gitian.sigs/ -r ${VERSION}-osx-signed ../bitcoin/contrib/gitian-descriptors/gitian-osx-signer.yml
    mv build/out/bitcoin-osx-signed.dmg ../bitcoin-${VERSION}-osx.dmg
    popd

Create (and optionally verify) the signed Windows binaries:

    pushd ./gitian-builder
    ./bin/gbuild -i --commit signature=v${VERSION} ../bitcoin/contrib/gitian-descriptors/gitian-win-signer.yml
    ./bin/gsign --signer "$SIGNER" --release ${VERSION}-win-signed --destination ../gitian.sigs/ ../bitcoin/contrib/gitian-descriptors/gitian-win-signer.yml
    ./bin/gverify -v -d ../gitian.sigs/ -r ${VERSION}-win-signed ../bitcoin/contrib/gitian-descriptors/gitian-win-signer.yml
    mv build/out/bitcoin-*win64-setup.exe ../bitcoin-${VERSION}-win64-setup.exe
    popd

Commit your signature for the signed macOS/Windows binaries:

    pushd gitian.sigs
    git add ${VERSION}-osx-signed/"${SIGNER}"
    git add ${VERSION}-win-signed/"${SIGNER}"
    git commit -m "Add ${SIGNER} ${VERSION} signed binaries signatures"
    git push  # Assuming you can push to the gitian.sigs tree
    popd

### After 3 or more people have gitian-built and their results match:

- Create `SHA256SUMS.asc` for the builds, and GPG-sign it:

```bash
sha256sum * > SHA256SUMS
```

The list of files should be:
```
bitcoin-${VERSION}-aarch64-linux-gnu.tar.gz
bitcoin-${VERSION}-arm-linux-gnueabihf.tar.gz
bitcoin-${VERSION}-i686-pc-linux-gnu.tar.gz
bitcoin-${VERSION}-riscv64-linux-gnu.tar.gz
bitcoin-${VERSION}-x86_64-linux-gnu.tar.gz
bitcoin-${VERSION}-osx64.tar.gz
bitcoin-${VERSION}-osx.dmg
bitcoin-${VERSION}.tar.gz
bitcoin-${VERSION}-win64-setup.exe
bitcoin-${VERSION}-win64.zip
```
The `*-debug*` files generated by the gitian build contain debug symbols
for troubleshooting by developers. It is assumed that anyone that is interested
in debugging can run gitian to generate the files for themselves. To avoid
end-user confusion about which file to pick, as well as save storage
space *do not upload these to the bitcoin.org server, nor put them in the torrent*.

- GPG-sign it, delete the unsigned file:
```
gpg --digest-algo sha256 --clearsign SHA256SUMS # outputs SHA256SUMS.asc
rm SHA256SUMS
```
(the digest algorithm is forced to sha256 to avoid confusion of the `Hash:` header that GPG adds with the SHA256 used for the files)
Note: check that SHA256SUMS itself doesn't end up in SHA256SUMS, which is a spurious/nonsensical entry.

- Upload zips and installers, as well as `SHA256SUMS.asc` from last step, to the bitcoin.org server
  into `/var/www/bin/bitcoin-core-${VERSION}`

- A `.torrent` will appear in the directory after a few minutes. Optionally help seed this torrent. To get the `magnet:` URI use:
```bash
transmission-show -m <torrent file>
```
Insert the magnet URI into the announcement sent to mailing lists. This permits
people without access to `bitcoin.org` to download the binary distribution.
Also put it into the `optional_magnetlink:` slot in the YAML file for
bitcoin.org (see below for bitcoin.org update instructions).

- Update bitcoin.org version

  - First, check to see if the Bitcoin.org maintainers have prepared a
    release: https://github.com/bitcoin-dot-org/bitcoin.org/labels/Core

      - If they have, it will have previously failed their Travis CI
        checks because the final release files weren't uploaded.
        Trigger a Travis CI rebuild---if it passes, merge.

  - If they have not prepared a release, follow the Bitcoin.org release
    instructions: https://github.com/bitcoin-dot-org/bitcoin.org/blob/master/docs/adding-events-release-notes-and-alerts.md#release-notes

  - After the pull request is merged, the website will automatically show the newest version within 15 minutes, as well
    as update the OS download links. Ping @saivann/@harding (saivann/harding on Freenode) in case anything goes wrong

- Update other repositories and websites for new version

  - bitcoincore.org blog post

  - bitcoincore.org maintained versions update:
    [table](https://github.com/bitcoin-core/bitcoincore.org/commits/master/_includes/posts/maintenance-table.md)

  - bitcoincore.org RPC documentation update

  - Update packaging repo

      - Notify BlueMatt so that he can start building [the PPAs](https://launchpad.net/~bitcoin/+archive/ubuntu/bitcoin)

      - Create a new branch for the major release "0.xx" (used to build the snap package)

      - Notify MarcoFalke so that he can start building the snap package

        - https://code.launchpad.net/~bitcoin-core/bitcoin-core-snap/+git/packaging (Click "Import Now" to fetch the branch)
        - https://code.launchpad.net/~bitcoin-core/bitcoin-core-snap/+git/packaging/+ref/0.xx (Click "Create snap package")
        - Name it "bitcoin-core-snap-0.xx"
        - Leave owner and series as-is
        - Select architectures that are compiled via gitian
        - Leave "automatically build when branch changes" unticked
        - Tick "automatically upload to store"
        - Put "bitcoin-core" in the registered store package name field
        - Tick the "edge" box
        - Put "0.xx" in the track field
        - Click "create snap package"
        - Click "Request builds" for every new release on this branch (after updating the snapcraft.yml in the branch to reflect the latest gitian results)
        - Promote release on https://snapcraft.io/bitcoin-core/releases if it passes sanity checks

  - This repo

      - Archive the release notes for the new version to `doc/release-notes/` (branch `master` and branch of the release)

      - Create a [new GitHub release](https://github.com/bitcoin/bitcoin/releases/new) with a link to the archived release notes

      - Create a pinned meta-issue for testing the release candidate (see [this issue](https://github.com/bitcoin/bitcoin/issues/15555) for an example) and provide a link to it in the release announcements where useful

- Announce the release:

  - bitcoin-dev and bitcoin-core-dev mailing list

  - Bitcoin Core announcements list https://bitcoincore.org/en/list/announcements/join/

  - Update title of #bitcoin on Freenode IRC

  - Optionally twitter, reddit /r/Bitcoin, ... but this will usually sort out itself

  - Celebrate
