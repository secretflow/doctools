import petstoreSpec from '@readme/oas-examples/3.0/json/petstore.json';
import OAS from 'oas';

export const petstore = new OAS(JSON.stringify(petstoreSpec));
