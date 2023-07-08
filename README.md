<div align="center">

# chunkdrive

</div>

chunkdrive is a proof of concept tool that allows you to store vast amounts of data by splitting it into chunks and uploading them to services that offer free storage.

Each chunk is send to a random so called "bucket". Each bucket can be configured to use a different storage service and encryption method.

## Configuration

chunkdrive is configured using a YAML file, by default it looks for `config.yaml` in the current directory, but you can specify a different path using the `CD_CONFIG_PATH` environment variable.

<details>
<summary>Example config</summary>

```yaml
buckets:
  some_name_you_choose:
    source:
      type: local
      folder: /path/to/folder
      max_size: 1000000000 # optional
    encryption:
      type: aes
      key: your_encryption_key
  some_other_name_you_choose:
    source:
      type: discord_webhook
        url: https://discord.com/api/webhooks/1234567890/abcdefghijklmnopqrstuvwxyz
    encryption:  # if you want to use none, you can omit this section
      type: none
```

</details>

## Supported storage services

You can make as many buckets as you want, each bucket can have a different storage service or the same one.

<details>
<summary>Local folder</summary>

```yaml
buckets:
  some_name_you_choose:
    source:
      type: local
      folder: /path/to/folder
      max_size: 1000000000 # optional
```

</details>

<details>
<summary>Discord webhooks</summary>

```yaml
buckets:
  some_name_you_choose:
    source:
      type: discord_webhook
        url: https://discord.com/api/webhooks/1234567890/abcdefghijklmnopqrstuvwxyz
```

</details>

<details>
<summary>GitHub Releases</summary>

```yaml
buckets:
  some_name_you_choose:
    source:
      type: github_release
      user: your_github_username
      repo: your_github_repo
      pat: your_github_personal_access_token
```

`pat` should have the `repo` scope, so it can create releases and upload files to them.

</details>

## Debug shell

chunkdrive includes a debug shell that lets you inspect the state of the filesystem and the buckets. You can enter it by running `chunkdrive --shell`.