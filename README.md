# SSMConfigDaemon
Daemon for automatically handling configuration updates + template rendering from SSM 

Overview:
- Runs as a background (daemon) process, which can be executed within a docker container or standalone on EC2 Instance, etc.
- Parses config.json file and attempt to grab parameter store values based off of `"keys": []`
- Renders `"rendered_file"` based on `"template"` value (e.g., nginx template is rendered into an nginx.conf file)
- Reloads process based on `"reload_cmd"` (e.g., nginx -s reload)
- Templates will be re-rendered and reload_cmd executed

Automatic re-rendering:
- Acts as consumer for AWS Kinesis, which is receiving all parameter store events from event stream
- Any parameter store entries (keys) found within Kinesis that are defined in local config.json will trigger a re-render of templates

TODO:
- Have configuration flag to designate a list of keys as 'render once' vs 'render upon every parameter store key modification'
- Improve error handling (result types, etc.)
