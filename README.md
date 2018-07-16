# globalrefrates
Glabal Reference Rates project

Тестовая версия работает на базе Exonum - https://github.com/exonum

Описание транзакции

```javascript
{
  protocol_version: 0,
  service_id: 42, 
  message_id: 1,
  fields: [
    { name: 'ExchangeId', type: Exonum.Uint16 },
	{ name: 'ExchangeName', type: Exonum.String },
	{ name: 'TradeSymbol', type: Exonum.String },
	{ name: 'TradeAsset', type: Exonum.String },
	{ name: 'TradeCurrency', type: Exonum.String },
	{ name: 'TradeType', type: Exonum.String },
	{ name: 'TradeIsFiat', type: Exonum.Uint8 },
	
	{ name: 'TradeExchangeId', type: Exonum.String },
	{ name: 'TradeExchangeTs', type: Exonum.Uint64 },
	{ name: 'TradeExchangePrice', type: Exonum.Float64 },
	{ name: 'TradeExchangeAmount', type: Exonum.Float64 },
	{ name: 'TradeExchangeTotal', type: Exonum.Float64 },
	{ name: 'TradeExchangeDirection', type: Exonum.String },
	
	{ name: 'AdditionData', type: Exonum.String }
  ]
}
```

* message_id - для транзакций будет условно 1
* service_id - для сервиса который принимает трейды с бирж, код 42
* protocol_version - версия протокола 0 всегда

* ExchangeId - идентификатор биржи
* ExchangeName - название биржи (строка в нижнем регистре без спецсимволов)
* TradeSymbol - торговый символ, канонический (вида XXX/ZZZ)
* TradeAsset - код инструмента (XXX)
* TradeCurrency - код валюты (ZZZ)
* TradeType - тип инструмента (YYY, код согласно международной классификации типов, CUR (fx-spot), FUT, OPT etc.)
* TradeExchangeId - уникальный идентификатор сделки с биржи 
* TradeExchangeTs - unix timestamp котировки, в виде UTC + Milliseconds (если миллисекунд нет, 000 будет добавлено)
* TradeExchangePrice - цена из сделки (в бдущем перейти на целые числа)
* TradeExchangeAmount - объем сделки (в базовом активе) 
* TradeExchangeTotal  - обьем всей сделки (в валюте, если биржда не предоставляет этих данных, TradeExchangePrice*TradeExchangeAmount с округлением до 9 знака (todo: может до 18)
* TradeExchangeDirection - если биржа указывает, сделка BUY или SELL (BUY по умолчанию)
* AdditionData - произвольная дополнительная информация

Todo: в дальнейшем все словарные параметры вынести отдельно и указывать ID вместо строковых данных
