import {afterEach, beforeAll, describe, expect, jest, test} from '@jest/globals';

import { Configuration, Metadata } from '../../src'
import seaFetch from '../../src/api/seaFetch';

jest.mock("../../src/api/seaFetch", () => jest.fn());

import { mockServer } from './helper';

describe('Given Metadata API', () => {

  const config = new Configuration({ 
    apiKey: "test_apikey"
  })
  const metadata = new Metadata(config)  
  const server = mockServer("https://metadata.cplane.cloud/v1")

  afterEach(() => {
    seaFetch.mockClear()
  })

  test('get page returns one element', async () => {  
    server.get("/config?", {
      kvs: [
        {
          key: "Zm9v",
          value: "YmFy",
        }
      ],
      next_key: null,
    })

    expect(await metadata.getPage()).toStrictEqual({
        keyValuePairs: [
          {
            key: "foo",
            value: "bar",
          }
        ],
        nextKey: null,
    })
  });
  
  test('get a key-value pair', async () => {
    server.get("/config/base64:Zm9vL2Jhcg", {
      "key":"Zm9vL2Jhcg",
      "value":"dmFsdWU"
    })

    expect(await metadata.get({key: "foo/bar"})).toStrictEqual({
      key: "foo/bar",
      value: "value"
    })    
  })
  
  test('delete a key-value pair ', async () => {        
    server.delete("/config/base64:Zm9vL2Jhcg", "Ok")

    expect(await metadata.delete({key: "foo/bar"})).toBe(true)
  });

    
  test('set a key-value pair ', async () => {    
    server.put("/config/base64:YmFyL2Zvbw", "Ok")

    expect(await metadata.set({key: "bar/foo", value: "empty"})).toBe(true)
  });

  test('get page of directory ', async () => {
    server.get("/config/base64:Zm9v/?", {
      kvs: [
        {
          key: "Zm9v",
          value: "YmFy",
        }
      ],
      next_key: null,
    })     

    expect(await metadata.getPage({directory: {key: "foo"}})).toStrictEqual({
      keyValuePairs: [
        {
          key: "foo",
          value: "bar",
        }
      ],
      nextKey: null,
    })
  });

  test('get next page ', async () => {
    server.get("/config?from=base64%3AZm9v", {
      kvs: [
        {
          key: "Zm9v",
          value: "YmFy",
        }
      ],
      next_key: null,
    })   

    expect(await metadata.getPage({nextKey: { key: "foo" }})).toStrictEqual({
      keyValuePairs: [
        {
          key: "foo",
          value: "bar",
        }
      ],
      nextKey: null,
    })
  });


  test('get all pages ', async () => {
    server.get("/config?", {
      kvs: [
        {
          key: "Zm9v",
          value: "YmFy",
        }
      ],
      next_key: null,
    })      

    expect(await metadata.getAllPages()).toStrictEqual([{
      key: "foo",
      value: "bar",
    }]);
  });
  
});
