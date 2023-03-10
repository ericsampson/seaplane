import {afterEach, beforeAll, describe, expect, jest, test} from '@jest/globals';

import { Configuration, Compute } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());

const mockIdentify = (configuration: Configuration) => {
  seaFetch.mockImplementation((token: string) => ({
    post: (url: string, body: string) => Promise.resolve({ 
      ok: () => true,
      json: () => Promise.resolve({token: "test_token"}) 
    })
  }))
}

const postTokenMock = {
  post: (url: string, body: string) => Promise.resolve({ 
    ok: () => true,
    json: () => Promise.resolve({token: "test_token"}) 
  })
}

const textBody = (body: Object) => Promise.resolve({ 
  ok: () => true,
  text: () => Promise.resolve(body) 
})

const DEFAULT_FLIGHT = {
  oid: "frm-0ouz6ng05tvll000e14k2sd3og",
  name: "flight-name",
  image: "https://image-example.com"
}

const DEFAULT_FORMATION = {
  oid: "frm-0oug6ng05tvll000e14k2sd3og",
  name: "name",
  url: "https://url-example.com",
  flights: [DEFAULT_FLIGHT],
  gateway_flight: "gateway"
}

const API_DEFAULT_FORMATION = {
  "oid": DEFAULT_FORMATION.oid, 
  "name": DEFAULT_FORMATION.name, 
  "url": DEFAULT_FORMATION.url, 
  "flights": DEFAULT_FORMATION.flights, 
  "gateway-flight": DEFAULT_FORMATION.gateway_flight
}

describe('Given Compute API', () => {

  const config = new Configuration({ 
    apiKey: "test_apikey"
  })
  const compute = new Compute(config)  

  beforeAll(() => {
    mockIdentify(config)
  })

  afterEach(() => {
    seaFetch.mockClear()
  })

  test('get page returns one element', async () => {  
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => textBody({
        formations: [
          API_DEFAULT_FORMATION
        ],
        meta: {
          total: 1,
          prev: "https://prev-example.com",
          next: "https://next-example.com"
        },
        })
    }))    

      expect(await compute.getPage()).toStrictEqual({
        formations: [
          DEFAULT_FORMATION
        ],
        meta: {
          total: 1,
          prev: "https://prev-example.com",
          next: "https://next-example.com"
        }
      })
  });
  
  test('get a formation', async () => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => textBody(API_DEFAULT_FORMATION)
    }))    

    expect(await compute.get("frm-0oug6ng05tvll000e14k2sd3og")).toStrictEqual(DEFAULT_FORMATION)    
  })

  
  test('delete a formation by id ', async () => {    
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      delete: (url: string) => textBody("Ok")
    }))  

    await expect(compute.delete("frm-0oug6ng05tvll000e14k2sd3og")).resolves.not.toThrow()
  });

  
  test('create a formation ', async () => {    
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      post:(url: string, body: string) => textBody("Ok")
    })) 

    await expect(compute.create(DEFAULT_FORMATION)).resolves.not.toThrow()
  });

});
