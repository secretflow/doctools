import OAS from 'oas';
import petstoreSpec from '@readme/oas-examples/3.0/json/petstore.json';

export const petstore = new OAS(JSON.stringify(petstoreSpec));
