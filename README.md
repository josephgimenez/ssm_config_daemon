# SSMConfigDaemon
Daemon for automatically handling configuration from SSM via templates

SSMConfigDaemon will do the following:
- Runs as a background (daemon) process, which can be executed within a docker container.
- Parse config.json file and attempt to grab parameter store values based off of `"keys": []`
- Render `"rendered_file"` based on `"template"` value (e.g., nginx template is rendered into an nginx.conf file)
- Reload process based on `"reload_cmd"` (e.g., nginx -s reload)
- Listens for HTTP POST requests at `/config` and accepts JSON that specifies new template values
- template will be re-rendered and reload_cmd executed

Ideally this would accept HTTP POST requests from a Lambda, which would be triggered from parameter store update events.

TODO:
- Need to register ssm config daemon within dynamo/ssm with callback url, ssm keys to receive callbacks for, etc.
- Need lambda that receives ssm update events and triggers ssm config callback url with new SSM key values.
