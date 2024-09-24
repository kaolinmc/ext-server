V0.1.1

# Prerequisites

## Terms

 - **Extension Identifier**: The unique path in which extension versions are stored. More often than not, this will be a `group:artifact` combination like in Maven.
 - [**Extension Bundle**](#extension-bundle)
 - **Release Types**: The following are valid release types:
   - `release`
   - `beta`
   - `rc`\
   <strong>***</strong>Actual placement in the /registry route will not change based on release type unlike maven -SNAPSHOT versions.
 - [Extension Runtime Model](#extension-runtime-model)
   - [Partition Runtime Model](#partition-runtime-model)

# Routes

## Get Routes
 - /registry/***.\* &rarr; A file server matching ***.\* as a file, where extensions are stored
 - [/metadata](#metadata) &rarr; Repository metadata in the following Json Format
 - [/metadata/\<identifier>](#metadataidentifier) &rarr; Metadata about the given extension. Maintained by this repository.
 - [/registry/\<extension_identifier>/version/<name>-\<version>-metadata.json](#registryextension_identifierversionname-version-metadatajson) &rarr; Metadata about an extension version, maintained by the extension developers.
 - [/search?query=\<string>&page=\<int>&pagination=\<int>](#searchquerystringpageintpaginationint)

## Put Routes
 - [/registry](#registryidentifierversion) &rarr; Puts an extension bundle into the given identifier/version.
   - Authorization goes in the header as follows: `Authorization: Bearer <token>`, it should not be URL encrypted as is often the case with `Authorization: Basic...`. No user is needed, simply the auth token. 
# Responses

## Ok

### /metadata:
```json
{
   "name": "...",
   "description": "...",
   "icon": "<url or encoded or null>",
   "extension_count": 0,
   "app_ids": [
      "minecraft", "etc."
   ]
}
```
### /metadata/\<identifier>

```json
{
   "downloads": 0,
   "latest": {
      "release": "version" // or null
      "beta": "version" // or null
      "rc": "version" // or null
   },
   "versions": [
      {
         "version": "<version>-<release_type>",
         "release_type": "<release_type>",
         "metadata_path": "<path>"
      }
   ]
}
```

### /registry/\<extension_identifier>/version/<name>-\<version>-metadata.json

```json
{
   "name": "",
   "developers": [
      ""
   ],
   "icon": "<url or encoded or null>",
   "description": "",
   "tags": [
      ""
   ],
   "app": ""
}
```

### /search?query=\<string>&page=\<int>&pagination=\<int>

**Parameters**:
 - `query`: a string relating to the query of the search
 - `page`: the current page that the pagination is on
 - `pagination`: how many items to paginate per page.

**Response:**

```json
{
   "result": [
      { // Extension identifier
         "group": "<group>",
         "name": "<name>"
      }
   ]
}
```


### /registry

A successful request will simply return a 200 code. 



## Error

The following is the standard error response for all conforming repositories. All non-200 responses must contain this in the body. Error codes (such as NotFound) must be indicated by the response code.

```json
{
   "error_message": "",
   "details": "" // Or Null
}
```

# Extension Bundle

An extension bundle is a packaged extension used to push all contents to the repository at once for verification. The bundle conceptually is simply an archived (.zip) file with the following contents (internal paths must match as defined here):
All items will be added into the directory of the registry in a maven like pattern where the actual name in the bundle becomes the classifier and type of the file.
 - /erm.json *required
   - All partitions defined in the ERM must have an associated partition in the partitions directory. Partitions not defined in the ERM will not be published.
 - /metadata.json *required 
 - /\<partition>.jar
 - /\<partition>.json
 - /\<classifier>.\<type> - for example could be checksums. 


# Extension Runtime Model

ERM:
```json
{
  "apiVersion": 0,
  "groupId": "string",
  "name": "string",
  "version": "string",
  "repositories": [
    {
      // Custom repository declaration, actual type of `repositories` is a Map<String, String>
    }
  ],
  "parents": [
    // Extension parent
  ],
  "partitions": [
    // Extension partition reference
  ]
}
```

Extension parent: 
```json
{
  "group": "string",
  "extension": "string",
  "version": "string"
}
```

Extension partition reference:
```json
{
  "type": "string",
  "name": "string"
}
```

## Partition Runtime Model

```json
{
  "type": "string",
  "name": "string",
  "repositories": [
    {
      "type": "string",
      "settings": {
        // Custom repository declaration
      }
    }
  ],
  "dependencies": [
    {
      // Custom dependency declaration
    }
  ],
  "options": {
    // Custom options.
  }
}
```