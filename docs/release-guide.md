---
permalink: /docs/release-guide
---

# Release Guide

This document guide you through how to prepare and publish an official release.
Releasing a new version in the community contains multiple steps, here is an
overview.

[[toc]]

## Prepare the Release Notes

For a new release, you should first prepare the release note. Release note
contains summaries of new features, enhancements, bug fixes,
docs, known issues, and deprecation, etc. You can see the notes of
[past release](https://github.com/apache/incubator-teaclave/releases) as an example.

## Prepare the GPG Signing Key

You can skip this section if you have already uploaded your key. That is, the
GPG signing key has been added in the following places:
  - The KEYS file in repo (<https://github.com/apache/incubator-teaclave/blob/master/KEYS>)
  - Apache release dist sever (<https://dist.apache.org/repos/dist/release/incubator/teaclave/KEYS>)

If you are the first to publish a release, please follow this instruction to
generating and uplaoding keys.

To generate GPG key, please refer to
<https://www.apache.org/dev/openpgp.html#generate-key> for details.

If you want to do the release on another machine, you can transfer your GPG key
to that machine via the `gpg --export` and `gpg --import` commands.

The last step is to update the KEYS file with your code signing key. Check in
the changes to the main branch of the repository, as well as ASF SVN,

Here is an instruction of editing the KEYS file in the ASF SVN.

```
# the --depth=files will avoid checkout existing folders
svn co --depth=files "https://dist.apache.org/repos/dist/dev/incubator/teaclave" svn-dev-teaclave
cd svn-dev-teaclave
# edit KEYS file
svn ci --username $ASF_USERNAME --password "$ASF_PASSWORD" -m "Update KEYS"
# update downloads.apache.org
svn rm --username $ASF_USERNAME --password "$ASF_PASSWORD" https://dist.apache.org/repos/dist/release/incubator/teaclave/KEYS -m "Update KEYS"
svn cp --username $ASF_USERNAME --password "$ASF_PASSWORD" https://dist.apache.org/repos/dist/dev/incubator/teaclave/KEYS https://dist.apache.org/repos/dist/release/incubator/teaclave/ -m "Update KEYS"
```

## Cut a Release Candidate

To cut a release candidate, one needs to first cut a branch from the main branch
using selected version string. We follow the [semantic versioning](https://semver.org/)
guidelines for new version string. In short, x.y.z means MAJOR.MINOR.PATCH. Since we
already have the release note, we can decide the version string to be released
based on the changes.

Note that in our workflow, the main branch should be freezed during the
releasing period, i.e, no new features and enhancements can be merged into it.
Only changes on this release can be merged and committed into the releasing
branch.

For example, to release version 1.0.0, let us first create a new branch
`release-v1.0.0` from the main branch.

```
git clone https://github.com/apache/incubator-teaclave
cd incubator-teaclave
git checkout -b release-v1.0.0
```

The next step is to do a complete version bumping (e.g., changing files which
contain versions and bump them from v0.9.0 to v1.0.0). Note that this affect
multiple files in different languages. Then, commit the changes to this
releasing branch. Other bug fixes and docs improvements can be also commited at
this time.

When cleanups are done, make sure all tests can be passed. Then, add a tag with
the current commit in the form of "v1.0.0-rc.1" where 1 means it's the first
release candidate. You can add the tag using git or add on GitHub.

Using Git:
```
git tag v1.0.0-rc.1
git push origin v1.0.0-rc.1
```

## Create Release Artifacts

Create the source code artifacts, including a self-contained tarball without git
history, a signature file signed by keys in the KEYS file, and a sha256 hash
file.

```
git clone git@github.com:apache/incubator-teaclave.git apache-teaclave-1.0.0-rc.1-incubating
cd apache-teaclave-1.0.0-rc.1-incubating
git checkout v1.0.0-rc.1
mkdir build && cd build && cmake .. && cd ..    # This will init submodules and apply patches
rm -rf build
find . -name ".git" -print0 | xargs -0 rm -rf   # Remove all .git directories
cd ..
tar czvf apache-teaclave-1.0.0-rc.1-incubating.tar.gz apache-teaclave-1.0.0-rc.1-incubating
```

Use your GPG key to sign the created artifact. First make sure your GPG is set
to use the correct private key,

```
$ gpg --list-key
/home/user/.gnupg/pubring.kbx
------------------------------------
pub   rsa4096 2020-08-17 [SC]
      154xxx
uid           [ unknown] XXX (CODE SIGNING KEY) <xxx@apache.org>
sub   rsa4096 2020-08-17 [E]
```

```
gpg -u 154xxx --armor --detach-sign apache-teaclave-1.0.0-rc.1-incubating.tar.gz
sha512sum apache-teaclave-1.0.0-rc.1-incubating.tar.gz > apache-teaclave-1.0.0-rc.1-incubating.tar.gz.sha512
```

At this point, we got three files in the release artifacts:
  - `apache-teaclave-1.0.0-rc.1-incubating.tar.gz`: source code tarball
  - `apache-teaclave-1.0.0-rc.1-incubating.tar.gz.asc`: signature
  - `apache-teaclave-1.0.0-rc.1-incubating.tar.gz.sha512`: SHA512 hash

## Check the Artifacts

We suggest to double check the release artifacts, e.g., verify the signature,
hash value and build from scratch.
There is a [checklist](https://cwiki.apache.org/confluence/display/INCUBATOR/Incubator+Release+Checklist) which can help the process.

## Upload the Release Candidate Artifacts

The release artifacts needs to be uploaded to ASF SVN,

```
# the --depth=files will avoid checkout existing folders
svn co --depth=files "https://dist.apache.org/repos/dist/dev/incubator/teaclave" svn-dev-teaclave
cd svn-dev-teaclave
mkdir 1.0.0-rc.1
# copy files (.tar.gz, .asc, .sha512) into it
svn add 1.0.0-rc.1
svn ci --username $ASF_USERNAME --password "$ASF_PASSWORD" -m "Add 1.0.0-rc.1"
```

## Publish the Pre-Release on GitHub

The next step is to publish a pre-release. Go to the GitHub repo's "Releases"
tab and click "Draft a new release".

- Choose a tag and select v1.0.0-rc.1.
- Copy and paste the release note draft into the description box.
- Select "This is a pre-release".
- Upload the artifacts created by the previous steps.
- Click "Publish release".

## Call a Vote on the Release Candidate

There are two votes need to be done for releasing a incubating project.
The first vote takes place on the Apache Teaclave developers list
(`dev@teaclave.apache.org`). Once it is closed with pass, we can call for the
second in the Apache Incubator general list
(`general@incubator.apache.org`). Look at past voting threads to see how this
proceeds. The email should contains these information.

- Provide the link to the draft of the release notes in the email
- Provide the link to the release candidate artifacts
- Make sure the email is in text format and the links are correct

For the dev@ vote, there must be at least 3 binding +1 votes and more +1 votes
than -1 votes. Once the vote is done, you should also send out a summary email
with the totals, with a subject that looks something like `[VOTE][RESULT]`.

In ASF, votes are open at least 72hrs (3 days). If you don't get enough number
of binding votes within that time, you cannot close the voting deadline. You
need to extend it.

If the voting fails, the community needs to modified the release accordingly,
create a new release candidate and re-run the voting process.

Here are some examples:

**Vote in the dev@teaclave list**:

- subject: [VOTE] Release Apache Teaclave (incubating) v0.3.0-rc.1
- to: dev@teaclave.apache.org
- link: <https://lists.apache.org/thread/9dzwv0y9l9qf9hol2rpwv85ns1xfgn7k>

**Result in the dev@teaclave list**:

- subject: [RESULT][VOTE] Release Apache Teaclave (incubating) v0.3.0-rc.1
- to: dev@teaclave.apache.org
- link: <https://lists.apache.org/thread/tyqhx2m9q0z1qg7dbxczf58nnpvxfzrn>

**Vote in the general@incubator list**:

- subject: [VOTE] Release Apache Teaclave (incubating) v0.3.0-rc.1
- to: general@incubator.apache.org
- link: <https://lists.apache.org/thread/mrwl41shgx60p432mw2lc6zcdw1lk6lk>

**Result in the general@incubator list**:

- subject: [RESULT][VOTE] Release Apache Teaclave (incubating) v0.3.0-rc.1
- to: dev@teaclave.apache.org
- link: <https://lists.apache.org/thread/gbv3f7l9bf6t1876byqm1v4stsw7g00z>

## Post the Release

After the vote passes, we need to crate the final release artifacts:

```
cd svn-dev-teaclave
mkdir 1.0.0
# copy RC files (.tar.gz, .asc, .sha512) into it and rename them
cp 1.0.0-rc.1/apache-teaclave-1.0.0-rc.1-incubating.tar.gz 1.0.0/apache-teaclave-1.0.0-incubating.tar.gz
cp 1.0.0-rc.1/apache-teaclave-1.0.0-rc.1-incubating.tar.gz.asc 1.0.0/apache-teaclave-1.0.0-incubating.tar.gz.asc
cp 1.0.0-rc.1/apache-teaclave-1.0.0-rc.1-incubating.tar.gz.sha512 1.0.0/apache-teaclave-1.0.0-incubating.tar.gz.sha512
# edit the file name (i.e., remove the rc version) in the sha512 file
vi 1.0.0/apache-teaclave-1.0.0-incubating.tar.gz.sha512
svn add 1.0.0
svn ci --username $ASF_USERNAME --password "$ASF_PASSWORD" -m "Add 1.0.0"
```

To upload the binaries to Apache mirrors, you copy the binaries from the dev
directory (this should be where they are voted) to the dist directory.

```
export SVN_EDITOR=vim
svn cp https://dist.apache.org/repos/dist/dev/incubator/teaclave/1.0.0 https://dist.apache.org/repos/dist/release/incubator/teaclave/1.0.0

# If you've added your signing key to the KEYS file, also update the release copy.
svn co --depth=files "https://dist.apache.org/repos/dist/release/incubator/teaclave" svn-dist-teaclave
curl "https://dist.apache.org/repos/dist/dev/incubator/teaclave/KEYS" > svn-dist-teaclave/KEYS
(cd svn-dist-teaclave && svn ci --username $ASF_USERNAME --password "$ASF_PASSWORD" -m "Update KEYS")
```

Merge commits in the release branch to the main branch, create a new release tag
(v1.0.0 in this case) on Github and remove the pre-release candidate tag.

```
git checkout master
git merge release-v1.0.0 --ff-only
git tag v1.0.0
git push --delete origin v1.0.0-rc.1
git push --delete origin release-v1.0.0
```

At last update the release notes and corresponding artifacts on GitHub.

## Update the Website

The website repository is located at <https://github.com/apache/incubator-teaclave-website>.
Modify the download page to include the release artifacts as well as the GPG
signature and SHA hash. Note that only put the latest version in the download page.
Older releases are archived in the archive site automatically
(<https://archive.apache.org/dist/incubator/teaclave/>).

Note that the links to the release artifact should start with
`https://www.apache.org/dyn/closer.lua/incubator/teaclave` to better utilize the
Apache CND. You can refer to the previous release link.


## Post the Announcement

Post new version release annoucement to the mailing list, blog and other
channels (Twitter, Discord, etc.).

**Mailing list example**:
- subject: [ANNOUNCE] Apache Teaclave (incubating) 0.3.0 released
- to: announce@apache.org, dev@teaclave.apache.org
- link: <https://lists.apache.org/thread/frck6z5v135f8c7w64nkgqk86w1soqc7>

**Blog example**:
- title: Announcing Apache Teaclave (incubating) 0.3.0
- link: <https://teaclave.apache.org/blog/2021-10-01-announcing-teaclave-0-3-0/>
