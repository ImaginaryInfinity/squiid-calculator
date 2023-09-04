# Configuration Protocol

Squiid stores it's config file as [TOML](https://toml.io/en/), which organizes data into sections with key: value pairs. In order to manipulate Squiid's config file, you must make a request to the server with the type of `configuration`. The types of requests are documented below.

## Reading the config file
This section provides example requests and responses for the following config file:
```toml
[system]
start_mode = "algebraic"

[keybinds]
quit = "q"
mode_algebraic = "a"
```
----

### Get a specific key
  - Arguments:
    - `action_type`: (string) - a string of `get_key`
    - `section`: (string) - a string of the desired section
    - `key`: (string) - the key of which to fetch the value
  - Returns:
    - (any) A payload containing the value of the key. Can be any valid TOML type (string, array, bool, etc.)

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "get_key",
            "section": "system",
            "key": "start_mode"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": "algebraic"
    }
    ```

----

### List Sections

  - Arguments:
    - `action_type`: (`string`) - a string of `list_sections`.
  - Returns:
    - (`array[string]`) - A payload containing an array of strings of the section titles.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "list_sections"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": ["system", "keybinds"]
    }
    ```

----

### List Keys

  - Arguments:
    - `action_type`: (`string`) - a string of `list_keys`.
    - `section`: (`string`) - a string of the section name.
  - Returns:
    - (`array[string]`) - A payload containing an array of keys.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "list_keys",
            "section": "keybinds"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": ["quit", "mode_algebraic"]
    }
    ```

----

### List Values

  - Arguments:
    - `action_type`: (`string`) - a string of `list_values`.
    - `section`: (`string`) - a string of the section name.
  - Returns:
    - (`array[string]`) - A payload containing an array of the values.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "list_values",
            "section": "keybinds"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": ["q", "a"]
    }
    ```

----

### List Items

  - Arguments:
    - `action_type`: (`string`) - a string of `list_items`.
    - `section`: (`string`) - a string of the section name.
  - Returns:
    - (`array[array[string, string]]`) - A payload containing an array of key-value pairs.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "list_items",
            "section": "keybinds"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": [
            ["quit", "q"],
            ["mode_algebraic", "a"]
        ]
    }
    ```

----

## Writing the config file

If there is an error, you will get a `response_type` of `error` instead of `configuration`. More details [here](comm_prot.md#receiving-data-from-the-server).

### Set a specific key

  - Arguments:
    - `action_type`: (`string`) - a string of `set_key`.
    - `section`: (`string`) - a string of the section name.
    - `key`: (`string`) - a string of the key name.
    - `value`: (`any`) - the value to set. can be any valid TOML type.
  - Returns:
    - Nothing.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "set_key",
            "section": "system",
            "key": "start_mode",
            "value": "rpn"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": {}
    }
    ```

----

### Create a Section

  - Arguments:
    - `action_type`: (`string`) - a string of `create_section`.
    - `section`: (`string`) - a string of the section name.
  - Returns:
    - Nothing.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "create_section",
            "section": "my_section"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": {}
    }
    ```

----

### Delete a Section

  - Arguments:
    - `action_type`: (`string`) - a string of `delete_section`.
    - `section`: (`string`) - a string of the section name.
  - Returns:
    - Nothing.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "delete_section",
            "section": "my_section"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": {}
    }
    ```

----

### Delete a Key

  - Arguments:
    - `action_type`: (`string`) - a string of `delete_key`.
    - `section`: (`string`) - a string of the section name.
    - `key`: (`string`) - a string of the key name.
  - Returns:
    - Nothing.

=== "Request"

    ```json
    {
        "request_type": "configuration",
        "payload": {
            "action_type": "delete_key",
            "section": "my_section",
            "key": "my_key"
        }
    }
    ```

=== "Response"

    ```json
    {
        "response_type": "configuration",
        "payload": {}
    }
    ```