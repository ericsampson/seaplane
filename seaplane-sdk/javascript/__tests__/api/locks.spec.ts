import {afterEach, beforeAll, describe, expect, test, jest} from '@jest/globals';
import { Configuration, Locks } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());

import { mockIdentify, postTokenMock } from './helper';

const textBody = (body: Object) => Promise.resolve({ 
  ok: () => true,
  text: () => Promise.resolve(body) 
})

describe('Given Locks API', () => {

  const config = new Configuration({ 
    apiKey: "test_apikey"
  })  
  const locks = new Locks(config)

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
        "locks": [
            {
                "name": "bG9jay10ZXN0",
                "id": "BiqhSv0tuAk",
                "info": {"ttl": 1000, "client-id": "test", "ip": ""},
            }
        ],
        "next": null,
      })
    }))

    expect(await locks.getPage()).toStrictEqual({
        locks: [
          {
            id: "BiqhSv0tuAk",
            name: {
              name: "lock-test"
            },
            info: {ttl: 1000, clientId: "test", ip: ""},
          }
        ],
        nextLock: null,
      })
  });
  
  test('get a lock', async () => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => textBody({
        "name": "Zm9vL2Jhcg",
        "id": "BiqhSv0tuAk",
        "info": {"ttl": 1000, "client-id": "test", "ip": ""},
      })
    }))

    expect(await locks.get({name: "foo/bar"})).toStrictEqual({
      name: {
        name: "foo/bar"
      },
      id: "BiqhSv0tuAk",
      info: {ttl: 1000, "clientId": "test", ip: ""},
  })
    
  })

  test('acquire a lock', async () => {    
    seaFetch
      .mockReturnValue(postTokenMock)
      .mockReturnValueOnce({
        post: (url: string, body: string) => Promise.resolve({ 
          ok: () => true,
          text: () => Promise.resolve({"id": "AOEHFRa4Ayg", "sequencer": 3}) 
        })
      })

    expect(await locks.acquire({name: "foo/bar"}, "client-id", 60)).toStrictEqual({
      id: "AOEHFRa4Ayg", 
      sequencer: 3
    })
  });

  test('release a lock', async () => {    
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      delete: (url: string) => textBody("OK")
    }))

    expect(await locks.release({name: "foo/bar"}, "id")).toBe(true)
  });


  test('get page of directory ', async () => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => textBody({
        locks: [{
                name: "Zm9vL2Jhcg",
                id: "BiqhSv0tuAk",
                info: {"ttl": 1000, "client-id": "test", "ip": ""},
        }],
        next: null,
      })
    }))  

    expect(await locks.getPage({directory: {name: "foo/bar"}})).toStrictEqual({
      locks: [
        {
          id: "BiqhSv0tuAk",
          name: {
            name: "foo/bar"
          },
          info: {
            clientId: "test",
            ip: "",
            ttl: 1000,
          }
        }
      ],
      nextLock: null,
    })
  });

  
  test('get next page ', async () => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => textBody({
        locks: [
            {
                name: "Zm9vL2Jhcg",
                id: "BiqhSv0tuAk",
                info: {"ttl": 1000, "client-id": "test", "ip": ""},
            }
        ],
        next: null,
      })
    }))      

    expect(await locks.getPage({fromLock: {name: "foo"}})).toStrictEqual({
      locks: [
        {
          id: "BiqhSv0tuAk",
          name: {
            name: "foo/bar"
          },
          info: {
            clientId: "test",
            ip: "",
            ttl: 1000,
          }
        }
      ],
      nextLock: null,
    })
  });

  test('get all pages ', async () => {
    seaFetch.mockImplementation((token: string) => ({
      ...postTokenMock,
      get: (url: string) => textBody({
        locks: [
            {
                name: "Zm9vL2Jhcg",
                id: "BiqhSv0tuAk",
                info: {"ttl": 1000, "client-id": "test", "ip": ""},
            },
            {
              name: "Zm9v",
              id: "ASDF",
              info: {"ttl": 1000, "client-id": "test-id", "ip": ""},
          }
        ],
        next: null,
      })
    }))      

    expect(await locks.getAllPages()).toStrictEqual([
      {
      id: "BiqhSv0tuAk",
      name: {
        name: "foo/bar"
      },
      info: {
        clientId: "test",
        ip: "",
        ttl: 1000,
      }
    },{
      id: "ASDF",
      name: {
        name: "foo"
      },
      info: {
        clientId: "test-id",
        ip: "",
        ttl: 1000,
      }
    }]);
  });
  
});
