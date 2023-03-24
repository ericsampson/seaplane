import Configuration from '../configuration';
import Request from './request';
import seaFetch from './seaFetch';
import { Formation, FormationPage, toFormation, toFormationPage } from '../model/compute';

export default class Compute {
  url: string;
  request: Request;

  constructor(configuration: Configuration) {
    this.url = `${configuration.values().computeEndpoint}/formations`;
    this.request = new Request(configuration.identify);
  }

  async create(formation: Formation): Promise<void> {
    const gateway = formation.gateway_flight;
    delete formation['gateway_flight'];

    const payload = {
      ...formation,
      'gateway-flight': gateway,
    };

    await this.request.send((token) => seaFetch(token).post(this.url, JSON.stringify(payload)));
  }

  async get(formationId: string): Promise<Formation> {
    const url = `${this.url}/${formationId}`;

    const result = await this.request.send((token) => seaFetch(token).get(url));

    return toFormation(result);
  }

  async delete(formationId: string): Promise<void> {
    const url = `${this.url}/${formationId}`;

    await this.request.send((token) => seaFetch(token).delete(url));
  }

  async getPage(): Promise<FormationPage> {
    const result = await this.request.send((token) => seaFetch(token).get(this.url));

    return toFormationPage(result);
  }
}
