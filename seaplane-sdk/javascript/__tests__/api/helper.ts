import { jest, expect } from '@jest/globals';

import { Configuration } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());


const textBody = (body: Object) => Promise.resolve({ 
  ok: () => true,
  text: () => Promise.resolve(body) 
})

export const postTokenMock = {
  post: (url: string, body: string) => {
    expect(url).toBe("https://flightdeck.cplane.cloud/v1/token")

    return textBody({token: "test_token"})
  }
}

export const mockIdentify = (configuration: Configuration) => {  
  seaFetch.mockImplementation((token: string) => ({
    post: (url: string, body: string) => {                
      return Promise.resolve({ 
        ok: () => true,
        text: () => Promise.resolve({token: "test_token"}) 
      })
    }
  }))
}

export const mockServer = (serverUrl: string, auth: boolean = true)  => ({
  get: (path: string, body: Object) => {
    const authPost = auth ? postTokenMock : {}
    seaFetch.mockImplementation((token: string) => ({
      ...authPost,
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
  },
  post: (path: string, body: Object, returnBody: Object) => {
    if(auth) {      
      seaFetch
      .mockReturnValue(postTokenMock)
      .mockReturnValueOnce({
        post: (url: string, postBody: string) => {
          expect(url).toBe(serverUrl+path)
          expect(postBody).toBe(JSON.stringify(body))

          return textBody(returnBody)
        }
      })
    } else {
      seaFetch.mockImplementation((token: string) => ({
        post: (url: string, postBody: string) => {
          expect(url).toBe(serverUrl+path)
          expect(postBody).toBe(JSON.stringify(body))

          return textBody(returnBody)
        }
      }))
    }
  }
})
