import Configuration from '../configuration';
import seaFetch from './seaFetch';
import { log } from '../logging';
import { HTTPError } from '../model/errors';

export default class Identify {
  url: string;
  accessToken?: string;
  apiKey?: string;
  autoRenew: boolean;

  constructor(configuration: Configuration) {
    this.url = `${configuration.values().identifyEndpoint}/token`;
    this.apiKey = configuration.apiKey;
    this.accessToken = configuration.accessToken;
    this.autoRenew = configuration.autoRenew;
  }

  setUrl(url: string) {
    this.url = url;
  }

  setToken(accessToken: string) {
    this.autoRenew = false;
    this.accessToken = accessToken;
  }

  async getToken(): Promise<string> {
    log.info('Requestiong access token...');
    const json = {};
    const token = this.apiKey || '';
    const response = await seaFetch(token).post(this.url, JSON.stringify(json));

    if (response.ok) {
      const resBody = await response.text();
      const body = resBody as any;
      this.accessToken = body.token;
      return body.token;
    } else {
      const errorBody = await response.text();

      if (!this.apiKey) {
        log.error('API KEY not set, use sea.config.setApiKey');
      } else {
        this.accessToken = undefined;
        log.error(`Request access token exception with code ${response.status}, error ${errorBody}`);
      }
      throw new HTTPError(response.status, errorBody);
    }
  }

  async renewToken(): Promise<string> {
    this.accessToken = await this.getToken();
    return this.accessToken;
  }
}
