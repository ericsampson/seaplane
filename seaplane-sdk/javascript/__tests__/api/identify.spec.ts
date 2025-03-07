import {afterEach, beforeAll, describe, expect, jest, test} from '@jest/globals';

import { Configuration, Identify } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());

describe('Given Identify', () => {

  const config = new Configuration({ 
    apiKey: "test_apikey"
  })


  beforeAll(() => {
    
  })

  afterEach(() => {
    seaFetch.mockClear()
  })

  test('returns the token and save it locally', async() => {    
    seaFetch.mockImplementation((token: string) => ({
      post: (url: string, body: string) => Promise.resolve({ 
        ok: () => true,
        json: () => Promise.resolve({token: "test_token"}) 
      })
    }))

    const identify = new Identify(config)
    
    await identify.getToken()    

    expect(identify.accessToken).toBe("test_token")
  })

  test('autoRenew should be true', async() => {        
    const identify = new Identify(config)

    expect(identify.autoRenew).toBe(true)
  })

  test('autoRenew should be false when set the token', async() => {        
    const identify = new Identify(config)

    identify.setToken("this_is_a_token")

    expect(identify.autoRenew).toBe(false)
  })

  test('autoRenew should be false when set the token', async() => {        
    const identify = new Identify(config)

    identify.setToken("this_is_a_token")

    expect(identify.autoRenew).toBe(false)
  })

  test('accessToken should be the same as the set token', async() => {        
    const identify = new Identify(config)

    identify.setToken("this_is_a_token")

    expect(identify.accessToken).toBe("this_is_a_token")
  })

  test('accessToken should be the same as the set token', async() => {        
    seaFetch.mockImplementation((token: string) => ({
      post: (url: string, body: string) => Promise.resolve({ 
        ok: () => true,
        json: () => Promise.resolve({token: "renewed_token"}) 
      })
    }))

    const identify = new Identify(config)
        
    identify.setToken("this_is_a_token")    
    await identify.renewToken()
  
    expect(identify.accessToken).toBe("renewed_token")
  })
});