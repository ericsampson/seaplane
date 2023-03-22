import { jest, expect } from '@jest/globals';

import { Configuration } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());


export const postTokenMock = {
  post: (url: string, body: string) => {
    expect(url).toBe("https://flightdeck.cplane.cloud/v1/token")

    return Promise.resolve({ 
      ok: () => true,
      json: () => Promise.resolve({token: "test_token"}) 
    })
  }
}

const textBody = (body: Object) => Promise.resolve({ 
  ok: () => true,
  text: () => Promise.resolve(body) 
})

export const mockIdentify = (configuration: Configuration) => {  
  seaFetch.mockImplementation((token: string) => ({
    post: (url: string, body: string) => {                
      return Promise.resolve({ 
        ok: () => true,
        json: () => Promise.resolve({token: "test_token"}) 
      })
    }
  }))
}

export const mockServer = (serverUrl: string)  => ({
  get: (path: string, body: Object) => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => {
        expect(url).toBe(serverUrl+path)
        
        return textBody(body)
      }
    }))   
  },
  delete: (path: string, body: string) => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      delete: (url: string) => {
        expect(url).toBe(serverUrl+path)

        return textBody(body)
      }
    }))  
  },
  put: (path: string, body: string) => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      put: (url: string) => {
        expect(url).toBe(serverUrl+path)
        
        return textBody(body)
      }
    }))  
  }
})
