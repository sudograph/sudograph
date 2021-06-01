const Generator = require('yeoman-generator');

module.exports = class extends Generator {
    writing() {
        this.fs.copy(
            this.templatePath(),
            this.destinationPath()
        );

        this.fs.copy(
            this.templatePath('.*'),
            this.destinationPath()
        );
    }
};