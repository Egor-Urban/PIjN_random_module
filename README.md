# Random Module Microservice (PIjN Protocol)

## Description

Microservice for generating random strings and randomly selecting items. Used in the PIjN protocol project.

## Metadata

* **Developer**: Urban Egor
* **Random module version**: 4.5.38 a

## Startup

The server starts on IP `127.0.0.1` and a port obtained via an HTTP request to `http://127.0.0.1:1030/getport/random_module_microservice`.

## Endpoints

### POST `/generate_random_string`

Generates a random string.

#### JSON Parameters:

```json
{
  "use_digits": true,
  "use_lowercase": true,
  "use_uppercase": false,
  "use_spec": false,
  "length": 12
}
```

#### Constraints:

* `length`: 1 to 256
* At least one character type must be enabled

#### Response:

```json
{
  "success": true,
  "data": "aB9f4zL1..."
}
```

### POST `/generate_random_choose`

Randomly selects items from a list.

#### JSON Parameters:

```json
{
  "items": ["apple", "banana", "cherry"],
  "count": 2
}
```

#### Constraints:

* `count`: 1 to 100 and ≤ length of `items`

#### Response:

```json
{
  "success": true,
  "data": ["banana", "apple"]
}
```

## Module `random_module`

### `generate_random_string(...) -> String`

Generates a random string using specified rules.

Parameters:

* `use_digits`: `bool`
* `use_lowercase`: `bool`
* `use_uppercase`: `bool`
* `use_spec`: `bool`
* `length`: `usize`

### `generate_random_choose(items: Vec<T>, count: usize) -> Vec<T>`

Randomly selects items from the `items` vector.

Constraint: `count ≤ items.len()`

## Logging

All requests and events are logged to `./logs/random_module_microservice_<date>.log` with timestamp, source, and level.

## Libraries Used

* `actix_web`
* `serde`
* `rand`, `rand_chacha`
* `reqwest`
* `chrono`
* `once_cell`

## Note

* Confidential data is not stored.
* The module uses a cryptographically secure RNG: `ChaCha20Rng`.
