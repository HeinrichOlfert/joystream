### 3.0.0

- **Carthage release:** Breaking change for files upload endpoint, the parameters except the file itself i.e., `dataObjectId`, `storageBucketId` and `bagId` are removed from the request body and are now part of the query string. This allows the pre-validation of the request params to validate the request before the complete file is uploaded successfully,