import { Dayjs } from 'dayjs';

export interface OpValString {
  $eq?: string;
  $not?: string;
  $in?: string[];
  $notIn?: string[];
  $lt?: string;
  $lte?: string;
  $gt?: string;
  $gte?: string;
  $contains?: string;
  $notContains?: string;
  $containsAny?: string[];
  $notContainsAny?: string[];
  $containsAll?: string[];
  $notContainsAll?: string[];
  $startsWith?: string;
  $notStartsWith?: string;
  $startsWithAny?: string[];
  $notStartsWithAny?: string[];
  $endsWith?: string;
  $notEndsWith?: string;
  $endsWithAny?: string[];
  $notEndsWithAny?: string[];
  $empty?: boolean;
  $null?: boolean;
  $containsCi?: string;
  $notContainsCi?: string;
  $startsWithCi?: string;
  $notStartsWithCi?: string;
  $endsWithCi?: string;
  $notEndsWithCi?: string;
  $ilike?: string;
}

export type OpValsString = Record<string, OpValString>;

export interface OpValBool {
  $eq?: boolean;
  $not?: boolean;
  $null?: boolean;
}

export type OpValsBool = Record<string, OpValBool>;

export interface OpValNumber {
  $eq?: number;
  $not?: number;
  $in?: number[];
  $notIn?: number[];
  $lt?: number;
  $lte?: number;
  $gt?: number;
  $gte?: number;
  $null?: boolean;
}

export type OpValsNumber = Record<string, OpValNumber>;

export interface OpValDate {
  $eq?: Date;
  $not?: Date;
  $in?: Date[];
  $notIn?: Date[];
  $lt?: Date;
  $lte?: Date;
  $gt?: Date;
  $gte?: Date;
  $null?: boolean;
}

export type OpValsDate = Record<string, OpValDate>;

export interface OpValDayJs {
  $eq?: Dayjs;
  $not?: Dayjs;
  $in?: Dayjs[];
  $notIn?: Dayjs[];
  $lt?: Dayjs;
  $lte?: Dayjs;
  $gt?: Dayjs;
  $gte?: Dayjs;
  $null?: boolean;
}

export type OpValsDayJs = Record<string, OpValDayJs>;

export interface OpValDateTime {
  $eq?: string;
  $not?: string;
  $in?: string[];
  $notIn?: string[];
  $lt?: string;
  $lte?: string;
  $gt?: string;
  $gte?: string;
  $null?: boolean;
}

export type OpValsDateTime = Record<string, OpValDateTime>;

export interface OpValValue {
  $eq?: any;
  $not?: any;
  $in?: any[];
  $notIn?: any[];
  $lt?: any;
  $lte?: any;
  $gt?: any;
  $gte?: any;
  $null?: boolean;
}

export type OpValsValue = Record<string, OpValValue>;

export interface OpValUuid {
  $eq?: string;
  $not?: string;
  $in?: string[];
  $notIn?: string[];
  $lt?: string;
  $lte?: string;
  $gt?: string;
  $gte?: string;
  $null?: boolean;
}

export type OpValsUuid = Record<string, OpValUuid>;

export type ArrayType = string[] | number[] | boolean[] | Dayjs[] | Date[];

export interface ArrayOpVal {
  $eq?: ArrayType;
  $not?: ArrayType;
  $contains?: ArrayType;
  $contained?: ArrayType;
}

export type ArrayOpVals = Record<string, ArrayOpVal>;
