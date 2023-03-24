import {afterEach, beforeAll, describe, expect, test, jest} from '@jest/globals';
import { Configuration, Locks } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());

import { mockServer } from './helper';

const EMPTY_BODY = {}

describe('Given Locks API', () => {

  const config = new Configuration({ 
    apiKey: "test_apikey"
  })  
  const locks = new Locks(config)
  const server = mockServer("https://metadata.cplane.cloud/v1")  

  afterEach(() => {
    seaFetch.mockClear()
  })

  test('get page returns one element', async () => {      
    server.get("/locks?", {
      "locks": [
          {
              "name": "bG9jay10ZXN0",
              "id": "BiqhSv0tuAk",
              "info": {"ttl": 1000, "client-id": "test", "ip": ""},
          }
      ],
      "next": null,
    })

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
    server.get("/locks/base64:Zm9vL2Jhcg", {
      "name": "Zm9vL2Jhcg",
      "id": "BiqhSv0tuAk",
      "info": {"ttl": 1000, "client-id": "test", "ip": ""},
    })

    expect(await locks.get({name: "foo/bar"})).toStrictEqual({
      name: {
        name: "foo/bar"
      },
      id: "BiqhSv0tuAk",
      info: {ttl: 1000, "clientId": "test", ip: ""},
  })
    
  })

  test('acquire a lock', async () => {
    server.post("/locks/base64:Zm9vL2Jhcg?client-id=client-id&ttl=60", EMPTY_BODY, {
      "id": "AOEHFRa4Ayg", 
      "sequencer": 3
    })

    expect(await locks.acquire({name: "foo/bar"}, "client-id", 60)).toStrictEqual({
      id: "AOEHFRa4Ayg", 
      sequencer: 3
    })
  });

  test('release a lock', async () => {        
    server.delete("/locks/base64:Zm9vL2Jhcg?id=id", "OK")

    expect(await locks.release({name: "foo/bar"}, "id")).toBe(true)
  });


  test('get page of directory ', async () => {    
    server.get("/locks/base64:Zm9vL2Jhcg/?", {
      locks: [{
              name: "Zm9vL2Jhcg",
              id: "BiqhSv0tuAk",
              info: {"ttl": 1000, "client-id": "test", "ip": ""},
      }],
      next: null,
    }) 

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
    server.get("/locks?from=base64%3AZm9v", {
      locks: [
          {
              name: "Zm9vL2Jhcg",
              id: "BiqhSv0tuAk",
              info: {"ttl": 1000, "client-id": "test", "ip": ""},
          }
      ],
      next: null,
    })     

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
    server.get("/locks?", {
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
