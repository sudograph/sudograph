#!/usr/bin/env node

const yeoman = require('yeoman-environment');

let environment = yeoman.createEnv();

environment.register(require.resolve('generator-sudograph'));

environment.run('sudograph:app');