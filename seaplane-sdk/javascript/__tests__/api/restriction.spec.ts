import {afterEach, beforeAll, describe, expect, jest, test} from '@jest/globals';
import { Configuration, Restrictions } from '../../src'
import { Key } from '../../src/model/metadata'
import seaFetch from '../../src/api/seaFetch';
import { RestrictionState, SeaplaneApi } from '../../src/model/restrictions';

jest.mock("../../src/api/seaFetch", () => jest.fn());

import { postTokenMock } from './helper';

const textBody = (body: Object) => Promise.resolve({ 
  ok: () => true,
  text: () => Promise.resolve(body) 
})

const mockIdentify = (configuration: Configuration) => {
  seaFetch.mockImplementation((token: string) => (postTokenMock))
}

describe('Given Restrictions API', () => {

  const config = new Configuration({ 
    apiKey: "test_apikey"
  })
  const restrictions = new Restrictions(config)  

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
        "next_api": "locks",
        "next_key": "dGhlIG5leHQga2V5",
        "restrictions": [{
          "api": "config",
          "directory": "Zm9vL2Jhcgo",
          "details": {
            "regions_allowed": ["XE"],
            "regions_denied": ["XE"],
            "providers_allowed": ["AWS"],
            "providers_denied": ["AWS"]
          },
          "state": "enforced"
        }]
      })
    }))

    expect(await restrictions.getPage()).toStrictEqual({
      nextApi: SeaplaneApi.Locks,
      nextKey: { key: "the next key"},      
      restrictions: [{
        api: SeaplaneApi.Metadata,
        state: RestrictionState.Enforced,      
        directory: { key: "foo/bar\n"},
        details: {
          "regionsAllowed": ["XE"],
          "regionsDenied": ["XE"],
          "providersAllowed": ["AWS"],
          "providersDenied": ["AWS"]
        }        
      }]
    })
  });
  
  
});
